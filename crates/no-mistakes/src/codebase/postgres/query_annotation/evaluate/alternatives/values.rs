use super::super::Value;
use crate::fx::{FxHashMap, FxHashSet};

fn prefixes<'a>(value: &'a Value, found: &mut FxHashMap<u64, (&'a str, bool)>) {
    match value {
        Value::Prefix(text, complete, Some(id)) => {
            found.insert(*id, (text, *complete));
        }
        Value::Promise(value) => prefixes(value, found),
        Value::Aggregate(values) | Value::Arguments(_, values) => {
            for value in values {
                prefixes(value, found);
            }
        }
        _ => {}
    }
}

pub(super) fn changes(before: &Value, after: &Value, changed: &mut FxHashSet<u64>) {
    let mut previous = FxHashMap::default();
    let mut current = FxHashMap::default();
    prefixes(before, &mut previous);
    prefixes(after, &mut current);
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
