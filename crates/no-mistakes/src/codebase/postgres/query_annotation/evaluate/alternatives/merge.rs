use super::super::Value;
use crate::fx::{FxHashMap, FxHashSet};

fn alternatives(value: Value, values: &mut Vec<Value>) {
    if let Value::Aggregate(children) = value {
        for child in children {
            alternatives(child, values);
        }
    } else if !values.contains(&value) {
        values.push(value);
    }
}

pub(super) fn objects(
    joined: &mut FxHashMap<u64, Vec<Value>>,
    current: &FxHashMap<u64, Vec<Value>>,
) {
    for (id, after) in current {
        // Canonical module objects created in an arm can survive into later arms.
        // Current live IDs are joined; pruned objects are never resurrected.
        let after = after.clone();
        match joined.entry(*id) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(after);
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                for (before, after) in entry.get_mut().iter_mut().zip(after) {
                    if *before != after {
                        let mut values = Vec::new();
                        alternatives(before.clone(), &mut values);
                        alternatives(after, &mut values);
                        *before = Value::Aggregate(values);
                    }
                }
            }
        }
    }
}

pub(super) fn definite(
    common: &mut Option<FxHashSet<(u64, usize)>>,
    private: &mut FxHashSet<(u64, usize)>,
    current: &FxHashSet<(u64, usize)>,
    original: &FxHashMap<u64, Vec<Value>>,
) {
    // An arm-created identity exists only on that arm, so its deletions remain
    // definite whenever that object is returned. Shared objects need intersection.
    private.extend(
        current
            .iter()
            .filter(|(id, _)| !original.contains_key(id))
            .copied(),
    );
    let shared = current
        .iter()
        .filter(|(id, _)| original.contains_key(id))
        .copied()
        .collect::<FxHashSet<_>>();
    if let Some(common) = common {
        common.retain(|item| shared.contains(item));
    } else {
        *common = Some(shared);
    }
}

pub(super) fn value(before: Value, after: Value) -> Value {
    if before == after {
        return before;
    }
    let mut values = Vec::new();
    alternatives(before, &mut values);
    alternatives(after, &mut values);
    Value::Aggregate(values)
}
