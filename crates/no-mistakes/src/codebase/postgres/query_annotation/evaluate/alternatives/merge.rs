use super::super::Value;
use crate::fx::{FxHashMap, FxHashSet};

fn alternatives(value: Value, values: &mut Vec<Value>) {
    match value {
        // Flatten both wrappers so a nested branch join stays a flat candidate
        // list. Possible carries the implicit unknown that prefix projection
        // already treats as an unsafe-or-unproven outcome.
        Value::Aggregate(children) | Value::Possible(children) => {
            for child in children {
                alternatives(child, values);
            }
        }
        other => {
            if !values.contains(&other) {
                values.push(other);
            }
        }
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
                        // Differing dense slots are candidates, not an opaque
                        // aggregate. prefix() inspects Possible, so an
                        // unannotated arm stays a violation under ignore mode.
                        *before = Value::Possible(values);
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
    shared: &FxHashSet<u64>,
) {
    // An arm-created identity exists only on that arm, so its deletions remain
    // definite whenever that object is returned. Shared objects need intersection.
    private.extend(
        current
            .iter()
            .filter(|(id, _)| !shared.contains(id))
            .copied(),
    );
    let shared = current
        .iter()
        .filter(|(id, _)| shared.contains(id))
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
    // Sparse slots use the same candidate join as dense argument vectors.
    Value::Possible(values)
}

pub(super) fn shared_ids(
    objects: &FxHashMap<u64, Vec<Value>>,
    initial: &super::modules::Initials,
) -> FxHashSet<u64> {
    let mut shared = objects.keys().copied().collect::<FxHashSet<_>>();
    shared.extend(initial.arguments());
    shared
}
