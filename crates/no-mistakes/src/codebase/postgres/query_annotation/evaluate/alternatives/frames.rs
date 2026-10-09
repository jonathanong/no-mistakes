use super::super::{Environment, Value};
use crate::fx::{FxHashMap, FxHashSet};

fn environments(value: &Value, pending: &mut Vec<Environment>) {
    match value {
        Value::Function(_, _, env) => pending.push(*env),
        Value::Promise(value) => environments(value, pending),
        Value::Aggregate(values) | Value::Arguments(_, values) => {
            for value in values {
                environments(value, pending);
            }
        }
        _ => {}
    }
}

fn remap(value: &mut Value, indices: &FxHashMap<Environment, Environment>) {
    match value {
        Value::Function(_, _, env) => {
            if let Some(index) = indices.get(env) {
                *env = *index;
            }
        }
        Value::Promise(value) => remap(value, indices),
        Value::Aggregate(values) | Value::Arguments(_, values) => {
            for value in values {
                remap(value, indices);
            }
        }
        _ => {}
    }
}

/// Keep only speculative frames reachable from returned callbacks. Original
/// frames retain their identities; cycles in captured environments are bounded.
pub(super) fn compact(
    scopes: &mut Vec<FxHashMap<String, Value>>,
    original: usize,
    returned: &mut [Value],
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
) {
    let mut pending = Vec::new();
    for value in returned.iter() {
        environments(value, &mut pending);
    }
    let mut visited = FxHashSet::default();
    while let Some(env) = pending.pop() {
        if visited.insert(env) {
            for value in scopes[env].values() {
                environments(value, &mut pending);
            }
        }
    }
    let mut retained = visited
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

fn argument_ids(value: &Value, ids: &mut FxHashSet<u64>) {
    match value {
        Value::Arguments(id, values) => {
            ids.insert(*id);
            for value in values {
                argument_ids(value, ids);
            }
        }
        Value::Aggregate(values) => {
            for value in values {
                argument_ids(value, ids);
            }
        }
        Value::Promise(value) => argument_ids(value, ids),
        _ => {}
    }
}

/// Captured function environments are already included in the compact arena,
/// so scanning every retained frame covers indirect aliases without cycles.
pub(super) fn prune_deleted(
    scopes: &[FxHashMap<String, Value>],
    returned: &[Value],
    deleted: &mut FxHashSet<(u64, Option<usize>)>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
) {
    let mut ids = FxHashSet::default();
    for value in scopes
        .iter()
        .flat_map(|scope| scope.values())
        .chain(returned)
    {
        argument_ids(value, &mut ids);
    }
    // A retained parameter can own a disconnected slot after every arguments
    // alias is rebound. Its mapping still needs the definite deletion fact.
    for bindings in mapped.values() {
        ids.extend(bindings.iter().map(|(id, _)| *id));
    }
    deleted.retain(|(id, _)| ids.contains(id));
}
