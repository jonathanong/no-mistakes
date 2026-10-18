use super::super::{frame_arena::ScopeArena, Environment, Value};
use super::arena::ArenaMut;
use super::reachable;
use crate::fx::{FxHashMap, FxHashSet};

pub(super) struct Compaction {
    pub indices: FxHashMap<Environment, Environment>,
    pub arguments: FxHashSet<u64>,
    pub identities: FxHashSet<u64>,
    pub captured_changed: bool,
}

pub(in crate::codebase::postgres::query_annotation::evaluate) struct MutationState<'a> {
    pub deleted: &'a mut FxHashSet<(u64, Option<usize>)>,
    pub definite: &'a mut FxHashSet<(u64, usize)>,
    pub disconnected: &'a mut FxHashSet<(u64, usize)>,
    pub invalidated: &'a mut FxHashSet<u64>,
    pub updates: &'a mut FxHashMap<u64, Value>,
}

#[cfg(test)]
mod captures;
#[cfg(test)]
#[path = "frames/tests/remap.rs"]
mod remap;

fn compact_from_roots(
    scopes: &mut ScopeArena,
    roots: reachable::RootSeeds<'_>,
    returned: &[Value],
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    arena: ArenaMut<'_>,
    fresh: &mut FxHashMap<Environment, FxHashSet<String>>,
    captured: &mut FxHashMap<Environment, FxHashMap<String, Environment>>,
) -> Compaction {
    let live =
        reachable::collect_from_roots(scopes, roots, returned, arena.read(), mapped, captured);
    let dead = scopes
        .ids()
        .filter(|env| !live.environments.contains(env))
        .collect::<Vec<_>>();
    for env in dead {
        scopes.release(env);
    }
    arena.objects.retain(|id, _| live.arguments.contains(id));
    arena.extras.retain(|id, _| live.arguments.contains(id));
    mapped.retain(|env, _| live.environments.contains(env));
    fresh.retain(|env, _| live.environments.contains(env));
    let before = captured.len();
    captured.retain(|env, _| live.environments.contains(env));
    Compaction {
        indices: live.environments.iter().map(|env| (*env, *env)).collect(),
        arguments: live.arguments,
        identities: live.identities,
        captured_changed: before != captured.len(),
    }
}

mod state;
pub(super) use state::{compact_modules, prune_mutations};

#[cfg(test)]
mod identity_tests;
#[cfg(test)]
mod tests;
