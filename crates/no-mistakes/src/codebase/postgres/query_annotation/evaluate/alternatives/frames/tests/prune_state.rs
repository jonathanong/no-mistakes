use super::super::*;
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn prune_state(
    scopes: &ScopeArena,
    returned: &[Value],
    state: MutationState<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    arena: ArenaMut<'_>,
    captured: &FxHashMap<Environment, FxHashMap<String, Environment>>,
    transient: &[crate::codebase::postgres::query_annotation::evaluate::temporaries::TransientRoot],
) {
    if arena.objects.is_empty()
        && arena.extras.is_empty()
        && state.deleted.is_empty()
        && state.definite.is_empty()
        && state.disconnected.is_empty()
        && state.invalidated.is_empty()
        && state.updates.is_empty()
    {
        return;
    }
    let roots = scopes.ids().collect::<Vec<_>>();
    let live = reachable::collect_from_roots(
        scopes,
        reachable::RootSeeds {
            environments: &roots,
            transient,
            ..Default::default()
        },
        returned,
        arena.read(),
        mapped,
        captured,
    );
    arena.objects.retain(|id, _| live.arguments.contains(id));
    arena.extras.retain(|id, _| live.arguments.contains(id));
    prune_mutations(state, &live.arguments, &live.identities);
}
