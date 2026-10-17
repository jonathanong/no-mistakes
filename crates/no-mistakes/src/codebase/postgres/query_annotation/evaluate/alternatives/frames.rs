use super::super::{Environment, Value};
use super::arena::ArenaMut;
use super::{freshness, reachable};
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::fx::{FxHashMap, FxHashSet};

struct Roots<'a> {
    environments: &'a [Environment],
    arguments: &'a [u64],
    preserved: Environment,
    transient: &'a [super::super::temporaries::TransientRoot],
}

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

mod captures;
mod remap;
pub(super) use remap::remap;
use remap::remap_changed;

/// Keep only frames and heap objects reachable from restored scopes or callback
/// results. Original frame identities and all argument identities stay stable.
pub(super) struct ModuleRoots<'a> {
    pub original: usize,
    pub modules: &'a mut FxHashMap<std::path::PathBuf, Environment>,
    pub initials: &'a mut [super::modules::Initials],
    pub transient: &'a [super::super::temporaries::TransientRoot],
}

pub(super) fn compact(
    scopes: &mut Vec<Scope>,
    cache: ModuleRoots<'_>,
    returned: &mut [Value],
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    arena: ArenaMut<'_>,
    fresh: &mut FxHashMap<Environment, FxHashSet<String>>,
    captured: &mut FxHashMap<Environment, FxHashMap<String, Environment>>,
) -> Compaction {
    let original = cache.original;
    let mut roots = (0..original).collect::<Vec<_>>();
    roots.extend(cache.modules.values().copied());
    let mut argument_roots = Vec::new();
    for initial in cache.initials.iter() {
        roots.extend(initial.roots());
        argument_roots.extend(initial.arguments());
    }
    let indices = compact_from_roots(
        scopes,
        Roots {
            environments: &roots,
            arguments: &argument_roots,
            preserved: original,
            transient: cache.transient,
        },
        returned,
        mapped,
        arena,
        fresh,
        captured,
    );
    for initial in cache.initials.iter_mut() {
        initial.remap(&indices.indices);
    }
    for env in cache.modules.values_mut() {
        if let Some(remapped) = indices.indices.get(env) {
            *env = *remapped;
        }
    }
    indices
}

fn compact_from_roots(
    scopes: &mut Vec<Scope>,
    roots: Roots<'_>,
    returned: &mut [Value],
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    arena: ArenaMut<'_>,
    fresh: &mut FxHashMap<Environment, FxHashSet<String>>,
    captured: &mut FxHashMap<Environment, FxHashMap<String, Environment>>,
) -> Compaction {
    let reachable = reachable::collect_from_roots(
        scopes,
        reachable::RootSeeds {
            environments: roots.environments,
            arguments: roots.arguments,
            transient: roots.transient,
        },
        returned,
        arena.read(),
        mapped,
        captured,
    );
    let mut retained = reachable
        .environments
        .into_iter()
        .filter(|env| *env >= roots.preserved)
        .collect::<Vec<_>>();
    retained.sort_unstable();
    let indices = retained
        .iter()
        .enumerate()
        .map(|(index, old)| (*old, roots.preserved + index))
        .collect::<FxHashMap<_, _>>();
    let environments_changed = indices.iter().any(|(before, after)| before != after);
    let mut frames = retained
        .into_iter()
        .map(|env| std::mem::take(&mut scopes[env]))
        .collect::<Vec<_>>();
    // A mutable Scope view detaches shared bindings and clears their indexes.
    // Prune unreachable state below even when retained environments keep IDs.
    if environments_changed {
        for frame in &mut frames {
            for value in frame.values_mut() {
                remap_changed(value, &indices);
            }
        }
        for value in returned {
            remap_changed(value, &indices);
        }
    }
    arena
        .objects
        .retain(|id, _| reachable.arguments.contains(id));
    if environments_changed {
        for values in arena.objects.values_mut() {
            for value in values {
                remap_changed(value, &indices);
            }
        }
    }
    arena
        .extras
        .retain(|id, _| reachable.arguments.contains(id));
    if environments_changed {
        for slots in arena.extras.values_mut() {
            for value in slots.values_mut() {
                remap_changed(value, &indices);
            }
        }
    }
    *mapped = std::mem::take(mapped)
        .into_iter()
        .filter_map(|(env, value)| {
            if env < roots.preserved {
                Some((env, value))
            } else {
                indices.get(&env).map(|new| (*new, value))
            }
        })
        .collect();
    let captured_changed = captures::remap_after_compaction(
        captured,
        scopes.len(),
        roots.preserved,
        &indices,
        environments_changed,
        &mut std::mem::drop::<Environment>,
        &mut std::mem::drop::<Environment>,
    );
    freshness::remap(fresh, roots.preserved, &indices);
    scopes.truncate(roots.preserved);
    scopes.extend(frames);
    Compaction {
        indices,
        arguments: reachable.arguments,
        identities: reachable.identities,
        captured_changed,
    }
}

mod state;
pub(super) use state::{compact_modules, prune_mutations, prune_state};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod identity_tests;
