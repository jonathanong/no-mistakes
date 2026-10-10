use super::super::value::Values;
use super::super::{Environment, Value};
use super::arena::ArenaMut;
use super::{freshness, reachable};
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::fx::{FxHashMap, FxHashSet};

struct Roots<'a> {
    environments: &'a [Environment],
    preserved: Environment,
}

pub(in crate::codebase::postgres::query_annotation::evaluate) struct MutationState<'a> {
    pub deleted: &'a mut FxHashSet<(u64, Option<usize>)>,
    pub definite: &'a mut FxHashSet<(u64, usize)>,
    pub disconnected: &'a mut FxHashSet<(u64, usize)>,
    pub invalidated: &'a mut FxHashSet<u64>,
}

pub(super) fn remap(value: &mut Value, indices: &FxHashMap<Environment, Environment>) {
    if indices.iter().all(|(before, after)| before == after) {
        return;
    }
    match value {
        Value::Function(_, _, env) => {
            if let Some(index) = indices.get(env) {
                *env = *index;
            }
        }
        Value::Promise(value) | Value::Evaluated(value, _) => remap(value, indices),
        Value::Aggregate(values) | Value::Joined(values) | Value::Possible(values)
            if changes_environment(values, indices) =>
        {
            for value in values {
                remap(value, indices);
            }
        }
        _ => {}
    }
}

fn changes_environment(values: &Values, indices: &FxHashMap<Environment, Environment>) -> bool {
    values.contains_environment()
        && values
            .environment_indices()
            .iter()
            .any(|env| indices.get(env).is_some_and(|after| after != env))
}

/// Keep only frames and heap objects reachable from restored scopes or callback
/// results. Original frame identities and all argument identities stay stable.
pub(super) struct ModuleRoots<'a> {
    pub original: usize,
    pub modules: &'a mut FxHashMap<std::path::PathBuf, Environment>,
    pub initials: &'a mut [super::modules::Initials],
}

pub(super) fn compact(
    scopes: &mut Vec<Scope>,
    cache: ModuleRoots<'_>,
    returned: &mut [Value],
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    arena: ArenaMut<'_>,
    fresh: &mut FxHashMap<Environment, FxHashSet<String>>,
    captured: &mut FxHashMap<Environment, FxHashMap<String, Environment>>,
) -> FxHashMap<Environment, Environment> {
    let original = cache.original;
    let mut roots = (0..original).collect::<Vec<_>>();
    roots.extend(cache.modules.values().copied());
    let count = returned.len();
    let mut rooted_values = returned.to_vec();
    for initial in cache.initials.iter() {
        roots.extend(initial.roots());
        rooted_values.extend(initial.arguments().map(Value::Arguments));
    }
    let indices = compact_from_roots(
        scopes,
        Roots {
            environments: &roots,
            preserved: original,
        },
        &mut rooted_values,
        mapped,
        arena,
        fresh,
        captured,
    );
    returned.clone_from_slice(&rooted_values[..count]);
    for initial in cache.initials.iter_mut() {
        initial.remap(&indices);
    }
    for env in cache.modules.values_mut() {
        if let Some(remapped) = indices.get(env) {
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
) -> FxHashMap<Environment, Environment> {
    let reachable = reachable::collect_from_roots(
        scopes,
        roots.environments,
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
    arena
        .objects
        .retain(|id, _| reachable.arguments.contains(id));
    for values in arena.objects.values_mut() {
        for value in values {
            remap(value, &indices);
        }
    }
    arena
        .extras
        .retain(|id, _| reachable.arguments.contains(id));
    for slots in arena.extras.values_mut() {
        for value in slots.values_mut() {
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

mod state;
pub(super) use state::{compact_modules, prune_state};

#[cfg(test)]
mod tests;
