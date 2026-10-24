//! Frozen refresh semantics before the no-op guards; intentionally detaches records.
use super::{Evaluator, Resolve, Value};

pub(super) fn refresh(evaluator: &Evaluator<'_, Resolve>, values: &mut [Value]) {
    for value in values {
        refresh_value(evaluator, value);
    }
}

fn refresh_value(evaluator: &Evaluator<'_, Resolve>, value: &mut Value) {
    match value {
        Value::Prefix(_, _, Some(id)) => {
            if evaluator.invalidated_builders.contains(id) {
                *value = Value::Unknown;
            } else if let Some(current) = evaluator.builder_updates.get(id) {
                *value = current.clone();
            }
        }
        Value::Promise(inner) | Value::Evaluated(inner, _) => refresh_value(evaluator, inner),
        Value::Aggregate(values)
        | Value::References(values)
        | Value::Object(values)
        | Value::Record(values, _)
        | Value::Joined(values)
        | Value::Possible(values) => refresh(evaluator, values),
        _ => {}
    }
}
