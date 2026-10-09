use super::super::Value;
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

/// Joins preserve existing builder-prefix proof, but never infer a SQL prefix
/// from conditional immutable binding assignments.
pub(super) fn join(
    joined: &mut [FxHashMap<String, Value>],
    current: &[FxHashMap<String, Value>],
    original: &[FxHashMap<String, Value>],
) {
    for ((before, after), initial) in joined.iter_mut().zip(current).zip(original) {
        for (name, value) in before {
            let next = &after[name];
            if *value == *next {
                continue;
            }
            if let Value::Prefix(_, _, Some(id)) = &initial[name] {
                if annotated_builder(&initial[name], *id)
                    && annotated_builder(value, *id)
                    && annotated_builder(next, *id)
                {
                    *value = initial[name].clone();
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
