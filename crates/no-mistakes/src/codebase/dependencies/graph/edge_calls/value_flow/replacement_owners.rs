use super::*;
use crate::codebase::dependencies::extract::ValueReplacementFacts;

/// Closed owner proofs consume the canonical pass once. Source names alone do
/// not prove callable identity, and different execution contexts never add up.
pub(super) fn repeated_owners(
    path: &std::path::Path,
    index: &CallableFileIndex,
    facts: &ValueReplacementFacts,
    sites: &[&ResolvedCallSite],
    references: &[crate::codebase::dependencies::extract::FunctionCall],
) -> FxHashSet<CallableId> {
    let mut candidates: FxHashSet<_> = facts
        .writes
        .iter()
        .filter_map(|write| write.owner)
        .filter(|owner| {
            !facts.unsafe_owners.contains(owner)
                && !index.values.async_callables.contains(owner)
                && !index.values.generator_callables.contains(owner)
                && index
                    .values
                    .scopes
                    .get(owner)
                    .is_some_and(|scope| !index.exported_scopes.contains(scope))
        })
        .collect();
    // Other repository execution and unresolved callees would require proving
    // recursion, reentrancy and effects. They are outside this bounded proof.
    for site in sites.iter().filter(|site| site.file == path) {
        if let Some(owner) = site.caller_id {
            if site.invocation != InvocationKind::Call
                || site.has_unknown_target()
                || site.target_nodes().next().is_some()
            {
                candidates.remove(&owner);
            }
        }
    }
    let mut events: Vec<_> = sites
        .iter()
        .filter(|site| site.file == path && site.caller_id.is_none())
        .collect();
    events.sort_unstable_by_key(|site| site.offset);
    let mut evidence: FxHashMap<CallableId, (usize, usize, Option<usize>, bool)> = fx_map();
    let mut previous = None;
    let mut segment = 0;
    for site in events {
        let owner = witness_owner(path, facts, &candidates, site);
        let region = facts.calls.get(&site.offset).and_then(|call| call.region);
        let key = owner.map(|owner| (owner, region));
        if key != previous {
            segment += 1;
            previous = key;
        }
        if let Some(owner) = owner {
            let record = evidence.entry(owner).or_insert((segment, 0, region, false));
            record.1 += 1;
            record.3 |= record.0 != segment || record.2 != region;
        }
    }
    // Every value use must be accounted for by the proven canonical calls.
    // A saved/exported/passed owner is an escape, even if two direct calls also
    // appear next to one another. Counts come from the existing shared facts.
    let mut reads: FxHashMap<CallableId, usize> = fx_map();
    for reference in references {
        if let Some(owner) =
            index.resolve_local_callable_id(reference.callee_binding_scope, &reference.callee)
        {
            if candidates.contains(&owner) {
                *reads.entry(owner).or_default() += 1;
            }
        }
    }
    for site in sites {
        for target in site.target_nodes() {
            if let NodeId::Symbol {
                file,
                callable_id: Some(owner),
                ..
            } = target
            {
                if file.as_ref() == path
                    && candidates.contains(owner)
                    && witness_owner(path, facts, &candidates, site) != Some(*owner)
                {
                    if let Some(record) = evidence.get_mut(owner) {
                        record.3 = true;
                    }
                }
            }
        }
    }
    evidence
        .into_iter()
        .filter_map(|(owner, (_, count, region, invalid))| {
            let repeated_region = region
                .is_some_and(|id| facts.regions[id].repeated && facts.regions[id].owner.is_none());
            (!invalid && reads.get(&owner) == Some(&count) && (count >= 2 || repeated_region))
                .then_some(owner)
        })
        .collect()
}
fn witness_owner(
    path: &std::path::Path,
    facts: &ValueReplacementFacts,
    candidates: &FxHashSet<CallableId>,
    site: &ResolvedCallSite,
) -> Option<CallableId> {
    if site.invocation != InvocationKind::Call {
        return None;
    }
    let witness = facts.calls.get(&site.offset)?;
    if !witness.unconditional || witness.owner.is_some() || site.has_unknown_target() {
        return None;
    }
    let mut nodes = site.target_nodes();
    let NodeId::Symbol {
        file,
        callable_id: Some(owner),
        ..
    } = nodes.next()?
    else {
        return None;
    };
    (file.as_ref() == path && nodes.next().is_none() && candidates.contains(owner))
        .then_some(*owner)
}
