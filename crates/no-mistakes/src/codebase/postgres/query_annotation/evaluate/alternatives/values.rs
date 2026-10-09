use super::super::Value;
use super::arena::Arena;
use crate::fx::{FxHashMap, FxHashSet};

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
        Value::Aggregate(values) | Value::Possible(values) => {
            for value in values {
                prefixes(value, arena, visited, found, definite);
            }
        }
        Value::Arguments(id) if visited.insert(*id) => {
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

pub(super) fn changes(
    before: &Value,
    after: &Value,
    before_arena: Arena<'_>,
    after_arena: Arena<'_>,
    changed: &mut FxHashSet<u64>,
    definite: &FxHashSet<(u64, usize)>,
) {
    let mut previous = FxHashMap::default();
    let mut current = FxHashMap::default();
    prefixes(
        before,
        before_arena,
        &mut FxHashSet::default(),
        &mut previous,
        definite,
    );
    prefixes(
        after,
        after_arena,
        &mut FxHashSet::default(),
        &mut current,
        definite,
    );
    for (id, (text, complete)) in previous {
        // A reference disappearing from this binding is a rebind, not a mutation.
        // Opaque mutation of the referenced identity is tracked independently.
        let Some((other, done)) = current.get(&id) else {
            continue;
        };
        let unchanged = (text == *other && complete == *done)
            || (text.trim_start().starts_with("/*") && other.trim_start().starts_with("/*"));
        if !unchanged {
            changed.insert(id);
        }
    }
}

pub(super) fn apply_taint(scopes: &mut [FxHashMap<String, Value>], changed: &FxHashSet<u64>) {
    for scope in scopes {
        for value in scope.values_mut() {
            if let Value::Prefix(_, _, Some(id)) = value {
                if changed.contains(id) {
                    *value = Value::Unknown;
                }
            }
        }
    }
}
