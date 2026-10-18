use super::super::Value;
#[cfg(test)]
use crate::fx::{FxHashMap, FxHashSet};

fn alternatives(value: Value, values: &mut Vec<Value>) {
    match value {
        // Flatten both wrappers so a nested branch join stays a flat candidate
        // list. Possible carries the implicit unknown that prefix projection
        // already treats as an unsafe-or-unproven outcome.
        Value::Aggregate(children) | Value::Joined(children) | Value::Possible(children) => {
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

#[cfg(test)]
pub(super) fn objects(
    joined: &mut FxHashMap<u64, Vec<Value>>,
    current: &FxHashMap<u64, Vec<Value>>,
) {
    let mut copied_slots = 0;
    for (id, after) in current {
        // Canonical module objects created in an arm can survive into later arms.
        // Current live IDs are joined; pruned objects are never resurrected.
        match joined.entry(*id) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(after.clone());
                copied_slots += after.len();
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                if entry.get().len() != after.len()
                    || entry.get() == &[Value::Failure("conditional-collection-shape")]
                    || after == &[Value::Failure("conditional-collection-shape")]
                {
                    // A missing element is not an empty SQL fragment. Retain
                    // shape uncertainty rather than zip away a possible write.
                    *entry.get_mut() = vec![Value::Failure("conditional-collection-shape")];
                    continue;
                }
                for (before, after) in entry.get_mut().iter_mut().zip(after) {
                    if &*before != after {
                        let mut values = Vec::new();
                        alternatives(before.clone(), &mut values);
                        alternatives(after.clone(), &mut values);
                        copied_slots += 1;
                        // Differing dense slots are candidates, not an opaque
                        // aggregate. prefix() inspects Possible, so an
                        // unannotated arm stays a violation under ignore mode.
                        *before = Value::Possible(values.into());
                    }
                }
            }
        }
    }
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment(
            "postgres.annotation.incoming_dense_slots_copied",
            copied_slots as u64,
        );
    }
}

#[cfg(test)]
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
    Value::Possible(values.into())
}

#[cfg(test)]
mod tests;
