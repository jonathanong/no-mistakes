use super::*;
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn prune_state(
    scopes: &[FxHashMap<String, Value>],
    returned: &[Value],
    state: MutationState<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    arena: ArenaMut<'_>,
    captured: &FxHashMap<Environment, FxHashMap<String, Environment>>,
) {
    let reachable = reachable::collect(
        scopes,
        scopes.len(),
        returned,
        arena.read(),
        mapped,
        captured,
    );
    arena
        .objects
        .retain(|id, _| reachable.arguments.contains(id));
    arena
        .extras
        .retain(|id, _| reachable.arguments.contains(id));
    state
        .deleted
        .retain(|(id, _)| reachable.arguments.contains(id));
    state
        .definite
        .retain(|(id, _)| reachable.arguments.contains(id));
    state
        .disconnected
        .retain(|(id, _)| reachable.arguments.contains(id));
    state
        .invalidated
        .retain(|id| reachable.identities.contains(id));
}

/// Compact request state from explicit module roots, then remap cached module
/// identities so later lookups continue to address their initialized frames.
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn compact_modules(
    scopes: &mut Vec<FxHashMap<String, Value>>,
    modules: &mut FxHashMap<std::path::PathBuf, Environment>,
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    arena: ArenaMut<'_>,
    fresh: &mut FxHashMap<Environment, FxHashSet<String>>,
    captured: &mut FxHashMap<Environment, FxHashMap<String, Environment>>,
) {
    let roots = modules.values().copied().collect::<Vec<_>>();
    let indices = compact_from_roots(
        scopes,
        Roots {
            environments: &roots,
            preserved: 0,
        },
        &mut [],
        mapped,
        arena,
        fresh,
        captured,
    );
    for env in modules.values_mut() {
        if let Some(remapped) = indices.get(env) {
            *env = *remapped;
        }
    }
}
