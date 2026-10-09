use super::super::{Environment, Value};
use super::{freshness, reachable};
use crate::fx::{FxHashMap, FxHashSet};

struct Roots<'a> {
    environments: &'a [Environment],
    preserved: Environment,
}

pub(in crate::codebase::postgres::query_annotation::evaluate) struct MutationState<'a> {
    pub deleted: &'a mut FxHashSet<(u64, Option<usize>)>,
    pub definite: &'a mut FxHashSet<(u64, usize)>,
    pub invalidated: &'a mut FxHashSet<u64>,
}

fn remap(value: &mut Value, indices: &FxHashMap<Environment, Environment>) {
    match value {
        Value::Function(_, _, env) => {
            if let Some(index) = indices.get(env) {
                *env = *index;
            }
        }
        Value::Promise(value) | Value::Evaluated(value, _) => remap(value, indices),
        Value::Aggregate(values) => {
            for value in values {
                remap(value, indices);
            }
        }
        _ => {}
    }
}

/// Keep only frames and heap objects reachable from restored scopes or callback
/// results. Original frame identities and all argument identities stay stable.
pub(super) fn compact(
    scopes: &mut Vec<FxHashMap<String, Value>>,
    original: usize,
    returned: &mut [Value],
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    objects: &mut FxHashMap<u64, Vec<Value>>,
    fresh: &mut FxHashMap<Environment, FxHashSet<String>>,
    captured: &mut FxHashMap<Environment, FxHashMap<String, Environment>>,
) {
    let roots = (0..original).collect::<Vec<_>>();
    compact_from_roots(
        scopes,
        Roots {
            environments: &roots,
            preserved: original,
        },
        returned,
        mapped,
        objects,
        fresh,
        captured,
    );
}

fn compact_from_roots(
    scopes: &mut Vec<FxHashMap<String, Value>>,
    roots: Roots<'_>,
    returned: &mut [Value],
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    objects: &mut FxHashMap<u64, Vec<Value>>,
    fresh: &mut FxHashMap<Environment, FxHashSet<String>>,
    captured: &mut FxHashMap<Environment, FxHashMap<String, Environment>>,
) -> FxHashMap<Environment, Environment> {
    let reachable = reachable::collect_from_roots(
        scopes,
        roots.environments,
        returned,
        objects,
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
    let mut frames = retained
        .into_iter()
        .map(|env| std::mem::take(&mut scopes[env]))
        .collect::<Vec<_>>();
    for frame in &mut frames {
        for value in frame.values_mut() {
            remap(value, &indices);
        }
    }
    for value in returned {
        remap(value, &indices);
    }
    objects.retain(|id, _| reachable.arguments.contains(id));
    for values in objects.values_mut() {
        for value in values {
            remap(value, &indices);
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
    *captured = std::mem::take(captured)
        .into_iter()
        .filter_map(|(env, mut origins)| {
            let key = if env < roots.preserved {
                Some(env)
            } else {
                indices.get(&env).copied()
            }?;
            for origin in origins.values_mut() {
                if let Some(index) = indices.get(origin) {
                    *origin = *index;
                }
            }
            Some((key, origins))
        })
        .collect();
    freshness::remap(fresh, roots.preserved, &indices);
    scopes.truncate(roots.preserved);
    scopes.extend(frames);
    indices
}

pub(super) fn prune_state(
    scopes: &[FxHashMap<String, Value>],
    returned: &[Value],
    state: MutationState<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    objects: &mut FxHashMap<u64, Vec<Value>>,
    captured: &FxHashMap<Environment, FxHashMap<String, Environment>>,
) {
    let reachable = reachable::collect(scopes, scopes.len(), returned, objects, mapped, captured);
    objects.retain(|id, _| reachable.arguments.contains(id));
    state
        .deleted
        .retain(|(id, _)| reachable.arguments.contains(id));
    state
        .definite
        .retain(|(id, _)| reachable.arguments.contains(id));
    state
        .invalidated
        .retain(|id| reachable.identities.contains(id));
}

/// Compact request state from explicit module roots, then remap cached module
/// identities so later lookups continue to address their initialized frames.
pub(super) fn compact_modules(
    scopes: &mut Vec<FxHashMap<String, Value>>,
    modules: &mut FxHashMap<std::path::PathBuf, Environment>,
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    objects: &mut FxHashMap<u64, Vec<Value>>,
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
        objects,
        fresh,
        captured,
    );
    for env in modules.values_mut() {
        if let Some(remapped) = indices.get(env) {
            *env = *remapped;
        }
    }
}
