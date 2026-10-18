use crate::codebase::postgres::query_annotation::evaluate::Value;
use crate::fx::{FxHashMap, FxHashSet};

fn alternatives(value: Value, values: &mut Vec<Value>) {
    match value {
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

pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn objects(
    joined: &mut FxHashMap<u64, Vec<Value>>,
    current: &FxHashMap<u64, Vec<Value>>,
) {
    let mut copied_slots = 0;
    for (id, after) in current {
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
                    *entry.get_mut() = vec![Value::Failure("conditional-collection-shape")];
                    continue;
                }
                for (before, after) in entry.get_mut().iter_mut().zip(after) {
                    if &*before != after {
                        let mut values = Vec::new();
                        alternatives(before.clone(), &mut values);
                        alternatives(after.clone(), &mut values);
                        copied_slots += 1;
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

pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn definite(
    common: &mut Option<FxHashSet<(u64, usize)>>,
    private: &mut FxHashSet<(u64, usize)>,
    current: &FxHashSet<(u64, usize)>,
    shared: &FxHashSet<u64>,
) {
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
