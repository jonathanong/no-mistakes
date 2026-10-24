use super::super::Value;
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::fx::FxHashMap;

#[cfg(not(test))]
#[macro_use]
mod binding_scan_count;
#[cfg(test)]
#[path = "bindings/binding_scan_count/tests.rs"]
#[macro_use]
mod binding_scan_count;

fn scalar(value: &Value) -> bool {
    match value {
        Value::Unknown | Value::Prefix(_, _, None) => true,
        Value::Evaluated(inner, _) => scalar(inner),
        Value::Aggregate(values)
        | Value::References(values)
        | Value::Object(values)
        | Value::Record(values, _)
        | Value::Joined(values) => values.iter().all(scalar),
        _ => false,
    }
}

fn annotated_builder(value: &Value, id: u64) -> bool {
    matches!(value, Value::Prefix(text, true, Some(current)) if *current == id && text.trim_start().starts_with("/*"))
}

fn candidates(value: Value, joined: &mut Vec<Value>) {
    match value {
        Value::Joined(values) => {
            for value in values {
                candidates(value, joined);
            }
        }
        value => {
            if !joined.contains(&value) {
                joined.push(value);
            }
        }
    }
}

const UNPROVEN: Value = Value::Unknown;

/// A binding absent from one arm is not SQL proof. Callers compare this instead
/// of indexing a divergent scope.
pub(super) fn binding_or_unproven<'a>(
    scope: &'a FxHashMap<String, Value>,
    name: &str,
) -> &'a Value {
    scope.get(name).unwrap_or(&UNPROVEN)
}

/// Joins preserve an annotated builder prefix only when the original scope and
/// both arms still have that builder. A missing arm or original binding is
/// unproven and must not keep or synthesize a SQL prefix.
pub(super) fn join_with_sql(
    joined: &mut [Scope],
    current: &[Scope],
    original: &[Scope],
    whole_sql: bool,
) {
    legacy_join!(joined, current, original, whole_sql);
    for ((before, after), initial) in joined.iter_mut().zip(current).zip(original) {
        // All three snapshots must agree: a name missing initially is unproven.
        if before.same_bindings(after) && before.same_bindings(initial) {
            continue;
        }
        // Inspect shared snapshots before taking a mutable view: unchanged
        // branches must not clone every binding in the captured environment.
        let dirty_candidates = if before.has_binding_changes() || after.has_binding_changes() {
            before.changed_binding_names(initial).and_then(|mut names| {
                names.extend(after.changed_binding_names(initial)?);
                Some(names)
            })
        } else {
            None
        };
        let changed = if let Some(names) = dirty_candidates {
            names
                .into_iter()
                .filter(|name| {
                    let Some(value) = before.get(name) else {
                        return false;
                    };
                    record_join_binding!();
                    after.get(name) != Some(value) || !initial.contains_key(name)
                })
                .collect::<Vec<_>>()
        } else {
            before
                .iter()
                .filter(|(name, value)| {
                    record_join_binding!();
                    after.get(*name) != Some(*value) || !initial.contains_key(*name)
                })
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>()
        };
        for name in changed {
            let value = before.get_mut(&name).expect("existing binding");
            // Divergent arms add and drop names. Neither hole is a builder.
            let Some(next) = after.get(&name) else {
                *value = Value::Unknown;
                continue;
            };
            let Some(initial_value) = initial.get(&name) else {
                *value = Value::Unknown;
                continue;
            };
            if let Value::Prefix(_, _, Some(id)) = initial_value {
                if !whole_sql
                    && annotated_builder(initial_value, *id)
                    && annotated_builder(value, *id)
                    && annotated_builder(next, *id)
                {
                    *value = initial_value.clone();
                    continue;
                }
            }
            *value = if !whole_sql && scalar(value) && scalar(next) {
                Value::Unknown
            } else {
                let mut values = Vec::new();
                candidates(std::mem::replace(value, Value::Unknown), &mut values);
                candidates(next.clone(), &mut values);
                super::super::concat::joined(values)
            };
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod fixture_tests;
#[cfg(test)]
mod identity_tests;
#[cfg(test)]
#[path = "bindings/tests/legacy.rs"]
mod legacy;
