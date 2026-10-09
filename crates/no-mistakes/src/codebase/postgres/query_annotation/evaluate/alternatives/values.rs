use super::super::Value;
use crate::fx::{FxHashMap, FxHashSet};

fn prefixes<'a>(
    value: &'a Value,
    objects: &'a FxHashMap<u64, Vec<Value>>,
    visited: &mut FxHashSet<u64>,
    found: &mut FxHashMap<u64, (&'a str, bool)>,
    definite: &FxHashSet<(u64, usize)>,
) {
    match value {
        Value::Prefix(text, complete, Some(id)) => {
            found.insert(*id, (text, *complete));
        }
        Value::Promise(value) => prefixes(value, objects, visited, found, definite),
        Value::Aggregate(values) => {
            for value in values {
                prefixes(value, objects, visited, found, definite);
            }
        }
        Value::Arguments(id) if visited.insert(*id) => {
            for (index, value) in objects[id].iter().enumerate() {
                // Removing a reference does not mutate the builder it held.
                if !definite.contains(&(*id, index)) {
                    prefixes(value, objects, visited, found, definite);
                }
            }
        }
        _ => {}
    }
}

pub(super) fn changes(
    before: &Value,
    after: &Value,
    before_objects: &FxHashMap<u64, Vec<Value>>,
    after_objects: &FxHashMap<u64, Vec<Value>>,
    changed: &mut FxHashSet<u64>,
    definite: &FxHashSet<(u64, usize)>,
) {
    let mut previous = FxHashMap::default();
    let mut current = FxHashMap::default();
    prefixes(
        before,
        before_objects,
        &mut FxHashSet::default(),
        &mut previous,
        definite,
    );
    prefixes(
        after,
        after_objects,
        &mut FxHashSet::default(),
        &mut current,
        definite,
    );
    for (id, (text, complete)) in previous {
        let unchanged = current.get(&id).is_some_and(|(other, done)| {
            (text == *other && complete == *done)
                || (text.trim_start().starts_with("/*") && other.trim_start().starts_with("/*"))
        });
        if !unchanged {
            changed.insert(id);
        }
    }
}
