use super::super::Value;
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::fx::FxHashMap;

fn scalar(value: &Value) -> bool {
    match value {
        Value::Unknown | Value::Prefix(_, _, None) => true,
        Value::Aggregate(values) => values.iter().all(scalar),
        _ => false,
    }
}

fn annotated_builder(value: &Value, id: u64) -> bool {
    matches!(value, Value::Prefix(text, true, Some(current)) if *current == id && text.trim_start().starts_with("/*"))
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
pub(super) fn join(joined: &mut [Scope], current: &[Scope], original: &[Scope]) {
    for ((before, after), initial) in joined.iter_mut().zip(current).zip(original) {
        for (name, value) in before {
            // Divergent arms add and drop names. Neither hole is a builder.
            let Some(next) = after.get(name) else {
                *value = Value::Unknown;
                continue;
            };
            let Some(initial_value) = initial.get(name) else {
                *value = Value::Unknown;
                continue;
            };
            if *value == *next {
                continue;
            }
            if let Value::Prefix(_, _, Some(id)) = initial_value {
                if annotated_builder(initial_value, *id)
                    && annotated_builder(value, *id)
                    && annotated_builder(next, *id)
                {
                    *value = initial_value.clone();
                    continue;
                }
            }
            *value = if scalar(value) && scalar(next) {
                Value::Unknown
            } else {
                Value::Aggregate(vec![value.clone(), next.clone()])
            };
        }
    }
}

#[cfg(test)]
mod tests;
