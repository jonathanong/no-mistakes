use super::super::{Environment, Value};
use super::{freshness, reachable};
use crate::fx::{FxHashMap, FxHashSet};

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
    let reachable = reachable::collect(scopes, original, returned, objects, mapped, captured);
    let mut retained = reachable
        .environments
        .into_iter()
        .filter(|env| *env >= original)
        .collect::<Vec<_>>();
    retained.sort_unstable();
    let indices = retained
        .iter()
        .enumerate()
        .map(|(index, old)| (*old, original + index))
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
            if env < original {
                Some((env, value))
            } else {
                indices.get(&env).map(|new| (*new, value))
            }
        })
        .collect();
    *captured = std::mem::take(captured)
        .into_iter()
        .filter_map(|(env, mut origins)| {
            let key = if env < original {
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
    freshness::remap(fresh, original, &indices);
    scopes.truncate(original);
    scopes.extend(frames);
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

/// Compact temporary evaluation frames before request-local snapshots.
pub(in crate::codebase::postgres::query_annotation::evaluate) fn prune_unreachable(
    scopes: &mut Vec<FxHashMap<String, Value>>,
    original: usize,
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    objects: &mut FxHashMap<u64, Vec<Value>>,
    fresh: &mut FxHashMap<Environment, FxHashSet<String>>,
    state: MutationState<'_>,
    captured: &mut FxHashMap<Environment, FxHashMap<String, Environment>>,
) {
    compact(scopes, original, &mut [], mapped, objects, fresh, captured);
    prune_state(scopes, &[], state, mapped, objects, captured);
}
