use super::*;
use crate::codebase::postgres::query_annotation::evaluate::Scope;
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) struct FrameContext<'a>
{
    pub transient: &'a [super::super::super::temporaries::TransientRoot],
    pub modules: &'a FxHashMap<std::path::PathBuf, Environment>,
}
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn prune_state(
    scopes: &[Scope],
    returned: &[Value],
    state: MutationState<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    arena: ArenaMut<'_>,
    captured: &FxHashMap<Environment, FxHashMap<String, Environment>>,
    context: FrameContext<'_>,
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
    let reachable = reachable::collect_from_roots(
        scopes,
        reachable::RootSeeds {
            modules: context.modules,
            environments: &roots,
            transient: context.transient,
            ..reachable::RootSeeds::all_modules(context.modules)
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
    modules: &mut FxHashMap<std::path::PathBuf, Environment>,
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    arena: ArenaMut<'_>,
    fresh: &mut FxHashMap<Environment, FxHashSet<String>>,
    captured: &mut FxHashMap<Environment, FxHashMap<String, Environment>>,
) -> Compaction {
    let indices = compact_from_roots(
        scopes,
        Roots {
            modules,
            module_policy: reachable::ModulePolicy::AllModules,
            environments: &[],
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
