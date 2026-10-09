use super::super::{Environment, Value};
use super::reachable;
use crate::fx::{FxHashMap, FxHashSet};

fn remap(value: &mut Value, indices: &FxHashMap<Environment, Environment>) {
    match value {
        Value::Function(_, _, env) => {
            if let Some(index) = indices.get(env) {
                *env = *index;
            }
        }
        Value::Promise(value) => remap(value, indices),
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
) {
    let reachable = reachable::collect(scopes, original, returned, objects, mapped);
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
    scopes.truncate(original);
    scopes.extend(frames);
}

pub(super) fn prune_state(
    scopes: &[FxHashMap<String, Value>],
    returned: &[Value],
    deleted: &mut FxHashSet<(u64, Option<usize>)>,
    definite: &mut FxHashSet<(u64, usize)>,
    invalidated: &mut FxHashSet<u64>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    objects: &mut FxHashMap<u64, Vec<Value>>,
) {
    let reachable = reachable::collect(scopes, scopes.len(), returned, objects, mapped);
    objects.retain(|id, _| reachable.arguments.contains(id));
    deleted.retain(|(id, _)| reachable.arguments.contains(id));
    definite.retain(|(id, _)| reachable.arguments.contains(id));
    invalidated.retain(|id| reachable.identities.contains(id));
}
