use super::*;

pub(super) struct CallAdmission {
    // (Some(owner), None) blocks every group of that file-local owner.
    pub(super) blocked: FxHashSet<(Option<CallableId>, Option<usize>)>,
    #[cfg(feature = "test-instrumentation")]
    pub(super) checks: usize,
    #[cfg(feature = "test-instrumentation")]
    pub(super) steps: usize,
}
struct Proof<'a> {
    file: &'a CallableFileIndex,
    receivers: &'a FxHashMap<String, usize>,
    writes: &'a index::PreparedReplacements,
    snapshots: &'a FxHashMap<CallableId, (bool, bool)>,
    #[cfg(feature = "test-instrumentation")]
    steps: usize,
}
/// This admits only the new repeated-region/owner proof. Ordinary source acquisition
/// still performs its existing timing and origin validation. All other calls
/// make that region opaque; external calls are not assumed to be pure.
pub(super) fn prepare(
    file: &CallableFileIndex,
    receivers: &FxHashMap<String, usize>,
    writes: &index::PreparedReplacements,
    snapshots: &FxHashMap<CallableId, (bool, bool)>,
    repeated: &FxHashSet<CallableId>,
    sites: &[&ResolvedCallSite],
) -> CallAdmission {
    let mut result = CallAdmission {
        blocked: fx_set(),
        #[cfg(feature = "test-instrumentation")]
        checks: 0,
        #[cfg(feature = "test-instrumentation")]
        steps: 0,
    };
    let mut proof = Proof {
        file,
        receivers,
        writes,
        snapshots,
        #[cfg(feature = "test-instrumentation")]
        steps: 0,
    };
    for site in sites {
        if !matches!(
            site.invocation,
            InvocationKind::Call
                | InvocationKind::Construct
                | InvocationKind::Get
                | InvocationKind::Set
        ) {
            continue;
        }
        let owner = site.caller_id;
        let region = writes.region_at(owner, site.offset);
        let owner_wide = owner.is_some_and(|owner| repeated.contains(&owner));
        if !owner_wide && !region.is_some_and(|region| writes.facts.regions[region].repeated) {
            continue;
        }
        // A nested region cannot hide execution from the enclosing owner's
        // next invocation. Module proofs remain local to independent regions.
        let blocked_region = if owner_wide { None } else { region };
        #[cfg(feature = "test-instrumentation")]
        {
            result.checks += 1;
        }
        let call = file
            .values
            .sites
            .get(&site.offset)
            .and_then(|sites| sites.get(&site.source_callee))
            .and_then(|key| file.values.calls.get(key));
        let admitted = site.invocation == InvocationKind::Call
            && call.is_some_and(|call| {
                !call.bound
                    && (proof.member(&call.callee, None, owner, region, 0)
                        || proof.getter(call, owner, region, 0))
            });
        if !admitted {
            result.blocked.insert((owner, blocked_region));
        }
    }
    #[cfg(feature = "test-instrumentation")]
    {
        result.steps = proof.steps;
    }
    result
}
impl Proof<'_> {
    // Bounded prepared-expression matching, not a second value resolver. Cyclic
    // aliases and unsupported projections decline the new closed-region proof.
    fn member(
        &mut self,
        value: &ValueExpression,
        property: Option<&str>,
        owner: Option<CallableId>,
        region: Option<usize>,
        depth: usize,
    ) -> bool {
        #[cfg(feature = "test-instrumentation")]
        {
            self.steps += 1;
        }
        if depth > 32 {
            return false;
        }
        let file = self.file;
        match value {
            ValueExpression::Named { name, scope, .. } => {
                let scope = scope.unwrap_or(0);
                let (name, property) = match name.split_once('.') {
                    Some((name, member)) if property.is_none() => (name, Some(member)),
                    Some(_) => return false,
                    None => (name.as_str(), property),
                };
                if let Some(binding) = scoped_name(&file.values.bindings, scope, name) {
                    return binding.invalidated_at.is_none()
                        && binding.mutation_contexts.is_empty()
                        && self.member(&binding.value, property, owner, region, depth + 1);
                }
                scope == 0
                    && property.is_some_and(|property| {
                        self.receivers.get(name).is_some_and(|receiver| {
                            self.writes
                                .tracks_member(*receiver, property, owner, region)
                                || owner.is_some()
                                    && region.is_some()
                                    && self.writes.tracks_member(*receiver, property, owner, None)
                        })
                    })
            }
            ValueExpression::Member(base, member) if property.is_none() => {
                self.member(base, Some(member), owner, region, depth + 1)
            }
            ValueExpression::Call(offset, end) if property.is_none() => file
                .values
                .calls
                .get(&(*offset, *end))
                .is_some_and(|call| self.getter(call, owner, region, depth + 1)),
            _ => false,
        }
    }
    fn getter(
        &mut self,
        call: &ValueCall,
        writer: Option<CallableId>,
        region: Option<usize>,
        depth: usize,
    ) -> bool {
        if depth > 32 || call.bound || !call.arguments.is_empty() || call.spread_at.is_some() {
            return false;
        }
        let owner = match &call.callee {
            ValueExpression::Callable(owner) => Some(*owner),
            ValueExpression::Named { name, scope, .. } => {
                self.file.resolve_local_callable_id(*scope, name)
            }
            _ => None,
        };
        let Some(owner) = owner.filter(|owner| self.snapshots.get(owner) == Some(&(true, true)))
        else {
            return false;
        };
        let file = self.file;
        file.values.returns.get(&owner).is_some_and(|returns| {
            returns.len() == 1 && self.member(&returns[0], None, writer, region, depth + 1)
        })
    }
}
