// Frozen pre-shortcut implementation from 273d01172; keep independent of the candidate.
use super::super::super::Value;
use crate::codebase::postgres::query_annotation::evaluate::Scope;

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

/// Joins preserve an annotated builder prefix only when the original scope and
/// both arms still have that builder. A missing arm or original binding is
/// unproven and must not keep or synthesize a SQL prefix.
pub(super) fn join_with_sql(
    joined: &mut [Scope],
    current: &[Scope],
    original: &[Scope],
    whole_sql: bool,
) {
    for ((before, after), initial) in joined.iter_mut().zip(current).zip(original) {
        // Inspect shared snapshots before taking a mutable view: unchanged
        // branches must not clone every binding in the captured environment.
        let changed = before
            .iter()
            .filter(|(name, value)| {
                super::binding_scan_count::record();
                after.get(*name) != Some(*value) || !initial.contains_key(*name)
            })
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
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
                super::super::super::concat::joined(values)
            };
        }
    }
}
