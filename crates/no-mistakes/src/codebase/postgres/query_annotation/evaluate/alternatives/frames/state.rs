use super::*;

#[cfg(test)]
#[path = "tests/prune_state.rs"]
mod legacy;
#[cfg(test)]
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) use legacy::prune_state;

pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn prune_mutations(
    state: MutationState<'_>,
    arguments: &FxHashSet<u64>,
    identities: &FxHashSet<u64>,
) {
    state.deleted.retain(|(id, _)| arguments.contains(id));
    state.definite.retain(|(id, _)| arguments.contains(id));
    state.disconnected.retain(|(id, _)| arguments.contains(id));
    state.invalidated.retain(|id| identities.contains(id));
    state.updates.retain(|id, _| identities.contains(id));
}

pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn compact_modules(
    scopes: &mut ScopeArena,
    modules: &mut FxHashMap<std::path::PathBuf, Environment>,
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    arena: ArenaMut<'_>,
    fresh: &mut FxHashMap<Environment, FxHashSet<String>>,
    captured: &mut FxHashMap<Environment, FxHashMap<String, Environment>>,
) -> Compaction {
    let roots = modules.values().copied().collect::<Vec<_>>();
    compact_from_roots(
        scopes,
        &roots,
        &[],
        &[],
        &[],
        mapped,
        arena,
        fresh,
        captured,
    )
}

#[cfg(test)]
mod tests;
