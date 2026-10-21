use super::super::replacement_values::{
    replacement_candidates, replacement_target, ReplacementCandidates,
};
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
        proof: (&FxHashSet<CallableId>, &super::calls::CallAdmission),
        foreign: &super::writers::ForeignWriters<'_>,
    ) -> Self {
        let (repeated, admission) = proof;
        #[cfg(feature = "test-instrumentation")]
        let mut probe = ReplacementProbeSnapshot::default();
        let leaves: Vec<_> = index
            .facts
            .writes
            .iter()
            .enumerate()
            .map(|(write_index, write)| {
                flow.default_origins = None;
                // A computed key may update another property. It supplies no
                // definite member value, either as a prefix or an entry tail.
                let admissible = write.property.is_some()
                    && write.unconditional
                    && stable_leaf(file, &write.value, 0)
                    && !unstable_rhs(
                        flow,
                        (file, path),
                        &write.value,
                        receivers,
                        index,
                        foreign,
                        0,
                    );
                let leaf = if admissible {
                    let value = flow.value(path, &write.value);
                    replacement_candidates(flow, value)
                } else {
                    ReplacementCandidates::Single(None)
                };
                #[cfg(feature = "test-instrumentation")]
                {
                    probe.rhs_attempts += 1;
                    probe.rhs_resolutions += usize::from(admissible);
                    probe.rhs_known += usize::from(leaf.has_known());
                    probe.rhs.push(ReplacementRhsProbe {
                        file: path.to_path_buf(),
                        write_index,
                        completed_at: write.completed_at,
                        owner: write.owner.map(|owner| owner.0),
                        region: write.region,
                        resolved: admissible,
                        known: leaf.has_known(),
                    });
                }
                #[cfg(not(feature = "test-instrumentation"))]
                let _ = write_index;
                leaf
            })
            .collect();
        let writes: Vec<_> = leaves.iter().cloned().map(replacement_target).collect();
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
            let opaque = admission.blocks(owner, region);
            #[cfg(all(test, feature = "test-instrumentation"))]
            let opaque = opaque && !flow.cache.opaque_entry_mode.original();
            #[cfg(feature = "test-instrumentation")]
            {
                probe.opaque_entry_resolutions_avoided += usize::from(!barrier && opaque);
            }
            // Admission is complete before payload construction. These entries
            // are hidden by target's blocked-first lookup, including prefixes.
            if barrier || opaque {
                blocked
                    .entry(group)
                    .or_default()
                    .insert(property.to_owned());
                continue;
            }
            let write = &index.facts.writes[first];
            let entry_offset = region.map_or(0, |id| index.facts.regions[id].start);
            flow.default_origins = None;
            #[cfg(feature = "test-instrumentation")]
            {
                probe.entry_resolutions += 1;
            }
            let original = flow.named(
                path,
                &format!("{}.{property}", write.receiver),
                Some(0),
                entry_offset,
                owner,
                None,
            );
            let original = replacement_candidates(flow, original);
            let tail = index.tail(receiver, property, owner, region).map_or_else(
                || ReplacementCandidates::Single(None),
                |tail| leaves[tail].clone(),
            );
            let target = replacement_target(original.into_iter().chain(tail));
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
    pub(super) fn block_regions(
        &mut self,
        index: &PreparedReplacements,
        admission: &super::calls::CallAdmission,
    ) {
        for (receiver, owner, region, property, _) in index.properties() {
            if admission.blocks(owner, region) {
                self.blocked
                    .entry((receiver, owner, region))
                    .or_default()
                    .insert(property.to_owned());
            }
        }
        #[cfg(feature = "test-instrumentation")]
        {
            self.probe.call_admission_checks += admission.checks;
            self.probe.call_proof_steps += admission.steps;
            self.probe.region_segment_searches += admission.region_searches;
            self.probe.opaque_groups += admission.blocked.len();
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
include!("replacement_stability.rs");
