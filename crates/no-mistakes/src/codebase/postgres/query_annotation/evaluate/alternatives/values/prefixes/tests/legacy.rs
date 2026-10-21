//! Frozen forward traversal: changing this reference would hide recall regressions.
use super::super::{Arena, Value};
use crate::fx::{FxHashMap, FxHashSet};

pub(super) fn collect<'a>(
    value: &'a Value,
    arena: Arena<'a>,
    definite: &FxHashSet<(u64, usize)>,
) -> FxHashMap<u64, (&'a str, bool)> {
    let mut found = FxHashMap::default();
    prefixes(
        value,
        arena,
        &mut FxHashSet::default(),
        &mut found,
        definite,
    );
    found
}

fn prefixes<'a>(
    value: &'a Value,
    arena: Arena<'a>,
    visited: &mut FxHashSet<u64>,
    found: &mut FxHashMap<u64, (&'a str, bool)>,
    definite: &FxHashSet<(u64, usize)>,
) {
    match value {
        Value::Prefix(text, complete, Some(id)) => {
            found.insert(*id, (text, *complete));
        }
        Value::Promise(value) | Value::Evaluated(value, _) => {
            prefixes(value, arena, visited, found, definite)
        }
        Value::Aggregate(values)
        | Value::References(values)
        | Value::Object(values)
        | Value::Record(values, _)
        | Value::Joined(values)
        | Value::Possible(values) => {
            // A reference-free shared subtree cannot affect either map or visited set.
            if !values.contains_reference() {
                return;
            }
            for value in values {
                prefixes(value, arena, visited, found, definite);
            }
        }
        Value::Arguments(id) | Value::Array(id) if visited.insert(*id) => {
            for (index, value) in arena.objects[id].iter().enumerate().chain(
                arena
                    .extras
                    .get(id)
                    .into_iter()
                    .flat_map(|slots| slots.iter().map(|(index, value)| (*index, value))),
            ) {
                // Removing a reference does not mutate the builder it held.
                if !definite.contains(&(*id, index)) {
                    prefixes(value, arena, visited, found, definite);
                }
            }
        }
        _ => {}
    }
}
