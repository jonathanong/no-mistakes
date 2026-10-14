use super::super::replacement_values::{replacement_candidate, replacement_target};
use super::index::PreparedReplacements;
use super::*;

type Group = (usize, Option<CallableId>, Option<usize>);
/// Targets are built at acquisition once. A reader borrows a normalized payload
/// after two prefix searches; it never allocates a frame or a candidate set.
pub(super) struct ReplacementPayloads {
    writes: Vec<ResolvedCallTarget>,
    entries: FxHashMap<Group, FxHashMap<String, ResolvedCallTarget>>,
    blocked: FxHashMap<Group, FxHashSet<String>>,
    #[cfg(feature = "test-instrumentation")]
    pub(super) probe: ReplacementProbeSnapshot,
}
impl ReplacementPayloads {
    pub(super) fn prepare(
        flow: &mut ValueFlowResolution<'_, '_>,
        path: &std::path::Path,
        file: &CallableFileIndex,
        receivers: &FxHashMap<String, usize>,
        index: &PreparedReplacements,
        repeated: &FxHashSet<CallableId>,
        foreign: &super::writers::ForeignWriters<'_>,
    ) -> Self {
        #[cfg(feature = "test-instrumentation")]
        let mut probe = ReplacementProbeSnapshot::default();
        let leaves: Vec<_> = index
            .facts
            .writes
            .iter()
            .enumerate()
            .map(|(write_index, write)| {
                flow.default_origins = None;
                let admissible = write.unconditional
                    && stable_leaf(file, &write.value, 0)
                    && !unstable_rhs(file, &write.value, receivers, index, foreign, path, 0);
                let leaf = if admissible {
                    let value = flow.value(path, &write.value);
                    replacement_candidate(flow, value)
                } else {
                    None
                };
                #[cfg(feature = "test-instrumentation")]
                {
                    probe.rhs_attempts += 1;
                    probe.rhs_resolutions += usize::from(admissible);
                    probe.rhs_known += usize::from(leaf.is_some());
                    probe.rhs.push(ReplacementRhsProbe {
                        file: path.to_path_buf(),
                        write_index,
                        completed_at: write.completed_at,
                        owner: write.owner.map(|owner| owner.0),
                        region: write.region,
                        resolved: admissible,
                        known: leaf.is_some(),
                    });
                }
                #[cfg(not(feature = "test-instrumentation"))]
                let _ = write_index;
                leaf
            })
            .collect();
        let writes: Vec<_> = leaves
            .iter()
            .cloned()
            .map(|value| replacement_target([value]))
            .collect();
        #[cfg(feature = "test-instrumentation")]
        for target in &writes {
            probe.target(target);
        }
        let mut entries: FxHashMap<Group, FxHashMap<String, ResolvedCallTarget>> = fx_map();
        let mut blocked: FxHashMap<Group, FxHashSet<String>> = fx_map();
        let mut aliases: FxHashMap<usize, Vec<&str>> = fx_map();
        for (name, receiver) in receivers {
            aliases.entry(*receiver).or_default().push(name);
        }
        for (receiver, owner, region, property, first) in index.properties() {
            let group = (receiver, owner, region);
            let supported = region.map_or_else(
                || owner.is_some_and(|owner| repeated.contains(&owner)),
                |id| index.facts.regions[id].repeated,
            );
            let barrier = !supported
                || foreign.contains(receiver, property, path)
                || index.reset(receiver)
                || aliases[&receiver]
                    .iter()
                    .any(|name| persistence_barrier(file, name, property, owner));
            if barrier {
                blocked
                    .entry(group)
                    .or_default()
                    .insert(property.to_owned());
                continue;
            }
            let write = &index.facts.writes[first];
            let entry_offset = region.map_or(0, |id| index.facts.regions[id].start);
            flow.default_origins = None;
            let original = flow.named(
                path,
                &format!("{}.{property}", write.receiver),
                Some(0),
                entry_offset,
                owner,
                None,
            );
            let original = replacement_candidate(flow, original);
            let tail = index
                .tail(receiver, property, owner, region)
                .and_then(|tail| leaves[tail].clone());
            let target = replacement_target([original, tail]);
            #[cfg(feature = "test-instrumentation")]
            probe.target(&target);
            entries
                .entry(group)
                .or_default()
                .insert(property.to_owned(), target);
        }
        Self {
            writes,
            entries,
            blocked,
            #[cfg(feature = "test-instrumentation")]
            probe,
        }
    }
    pub(super) fn target(
        &self,
        receiver: usize,
        property: &str,
        owner: Option<CallableId>,
        region: Option<usize>,
        prefix: Option<usize>,
    ) -> Option<&ResolvedCallTarget> {
        let group = (receiver, owner, region);
        if self
            .blocked
            .get(&group)
            .is_some_and(|properties| properties.contains(property))
        {
            return Some(&ResolvedCallTarget::Unknown);
        }
        if let Some(prefix) = prefix {
            return self.writes.get(prefix);
        }
        // Dynamic-only writes can affect every member, but supply no stable leaf.
        Some(
            self.entries
                .get(&group)
                .and_then(|properties| properties.get(property))
                .unwrap_or(&ResolvedCallTarget::Unknown),
        )
    }
}

fn persistence_barrier(
    file: &CallableFileIndex,
    name: &str,
    property: &str,
    owner: Option<CallableId>,
) -> bool {
    let Some(history) = file
        .values
        .receiver_mutations
        .as_ref()
        .and_then(|histories| histories.get(&0))
        .and_then(|names| names.get(name))
    else {
        return false;
    };
    // A reassigned receiver or another writer invalidates persistence. The
    // property-qualified summary includes omitted aliases without replaying it.
    history
        .dynamic
        .as_ref()
        .is_some_and(|history| history.summary.is_some())
        || history
            .members
            .get(property)
            .and_then(|history| history.summary.as_ref())
            .is_some_and(|summary| summary.contexts.iter().any(|writer| *writer != owner))
        || history
            .assigned
            .as_ref()
            .and_then(|assigned| assigned.members.get(property))
            .is_some_and(|write| write.multiple_contexts || write.context != owner)
        || history
            .assigned
            .as_ref()
            .and_then(|assigned| assigned.dynamic.as_ref())
            .is_some_and(|write| write.multiple_contexts || write.context != owner)
}

fn stable_leaf(file: &CallableFileIndex, expression: &ValueExpression, depth: usize) -> bool {
    if depth > 32 {
        return false;
    }
    match expression {
        ValueExpression::Callable(_) => true,
        ValueExpression::Member(base, _) => stable_leaf(file, base, depth + 1),
        ValueExpression::Named { name, scope, .. } => {
            let base = name.split_once('.').map_or(name.as_str(), |(base, _)| base);
            let scope = scope.unwrap_or(0);
            if let Some(binding) = scoped_name(&file.values.bindings, scope, base) {
                stable_leaf(file, &binding.value, depth + 1)
            } else {
                scoped_name(&file.values.parameters, scope, base).is_none()
                    && (scope == 0 && file.imported.contains_key(base)
                        || file.resolve_local_callable_id(Some(scope), base).is_some())
            }
        }
        _ => false,
    }
}
fn unstable_rhs(
    file: &CallableFileIndex,
    expression: &ValueExpression,
    receivers: &FxHashMap<String, usize>,
    index: &PreparedReplacements,
    foreign: &super::writers::ForeignWriters<'_>,
    path: &std::path::Path,
    depth: usize,
) -> bool {
    if depth > 32 {
        return true;
    }
    let (name, property, offset, owner) = match expression {
        ValueExpression::Named {
            name,
            scope,
            offset,
            context,
        } => match name.split_once('.') {
            Some((name, property)) => (name, property, *offset, *context),
            None => {
                return scoped_name(&file.values.bindings, scope.unwrap_or(0), name).is_some_and(
                    |binding| {
                        unstable_rhs(
                            file,
                            &binding.value,
                            receivers,
                            index,
                            foreign,
                            path,
                            depth + 1,
                        )
                    },
                );
            }
        },
        ValueExpression::Member(base, property) => match base.as_ref() {
            ValueExpression::Named {
                name,
                offset,
                context,
                ..
            } => (name.as_str(), property.as_str(), *offset, *context),
            _ => return true,
        },
        _ => return false,
    };
    let Some(receiver) = receivers.get(name) else {
        return true;
    };
    let repeated_acquisition = owner.is_some()
        || index
            .region_at(owner, offset)
            .is_some_and(|id| index.facts.regions[id].may_repeat);
    // A method captured once outside repetition remains a snapshot. A live
    // source acquired in the repeated body cannot inherit a later RHS value.
    foreign.contains(*receiver, property, path)
        || repeated_acquisition
            && (index.reset(*receiver)
                || index.repeat_mutated(*receiver, property)
                || persistence_barrier(file, name, property, owner))
}
