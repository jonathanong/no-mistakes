use super::*;
use crate::codebase::postgres::query_annotation::evaluate::{CapturedBindings, Scope};
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn prune_state(
    scopes: &[Scope],
    returned: &[Value],
    state: MutationState<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    arena: ArenaMut<'_>,
    captured: &CapturedBindings,
    transient: &[super::super::super::temporaries::TransientRoot],
) {
    // Scalar-only branches have no heap or mutation state to discard.
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
    let roots = (0..scopes.len()).collect::<Vec<_>>();
    // Peek at existing summaries; cold snapshots keep the shared raw visitor.
    let reachable = reachable::collect_warm_from_roots(
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
    arena
        .objects
        .retain(|id, _| reachable.arguments.contains(id));
    arena
        .extras
        .retain(|id, _| reachable.arguments.contains(id));
    prune_mutations(state, &reachable.arguments, &reachable.identities);
}

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

/// Compact request state from explicit module roots, then remap cached module
/// identities so later lookups continue to address their initialized frames.
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn compact_modules(
    scopes: &mut Vec<Scope>,
    modules: &mut FxHashMap<
        crate::codebase::postgres::query_annotation::module_ids::ModuleId,
        Environment,
    >,
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    arena: ArenaMut<'_>,
    fresh: &mut FxHashMap<Environment, FxHashSet<String>>,
    captured: &mut CapturedBindings,
) -> Compaction {
    let roots = modules.values().copied().collect::<Vec<_>>();
    let indices = compact_from_roots(
        scopes,
        Roots {
            environments: &roots,
            arguments: &[],
            preserved: 0,
            transient: &[],
        },
        &mut [],
        mapped,
        arena,
        fresh,
        captured,
    );
    for env in modules.values_mut() {
        if let Some(remapped) = indices.indices.get(env) {
            *env = *remapped;
        }
    }
    indices
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "state/summary_tests.rs"]
mod summary_tests;
