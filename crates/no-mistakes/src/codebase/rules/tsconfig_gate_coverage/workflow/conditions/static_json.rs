use super::StaticValue;
use serde_yaml::Value;

pub(super) fn literal_from_json_static_value(expression: &str) -> Option<StaticValue> {
    crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::literal_from_json_value(
        expression.trim(),
    )
    .map(static_yaml_value)
    .or_else(|| invalid_literal_from_json(expression).then_some(StaticValue::Invalid))
}

pub(super) fn invalid_literal_from_json(expression: &str) -> bool {
    crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::invalid_literal_from_json(
        &format!("${{{{ {expression} }}}}"),
    )
}

/// `toJSON` returns a pretty-printed JSON string. The static value model only
/// retains array contents, so mappings and nested non-stringable values remain
/// unresolved instead of inventing a serialized shape.
pub(super) fn to_json_static_value(value: StaticValue) -> Option<StaticValue> {
    match static_json_value(&value) {
        Err(()) => Some(StaticValue::Invalid),
        Ok(None) => None,
        Ok(Some(value)) => serde_json::to_string_pretty(&value)
            .ok()
            .map(StaticValue::String),
    }
}

fn static_json_value(value: &StaticValue) -> Result<Option<serde_json::Value>, ()> {
    match value {
        StaticValue::Bool(value) => Ok(Some(serde_json::Value::Bool(*value))),
        StaticValue::String(value) => Ok(Some(serde_json::Value::String(value.clone()))),
        StaticValue::Number(_) => value
            .format_string()
            .and_then(|value| serde_json::from_str(&value).ok())
            .map(Some)
            .ok_or(()),
        StaticValue::Null => Ok(Some(serde_json::Value::Null)),
        StaticValue::Sequence(values) => {
            let mut serialized = Vec::with_capacity(values.len());
            for value in values {
                match static_json_value(value)? {
                    Some(value) => serialized.push(value),
                    None => return Ok(None),
                }
            }
            Ok(Some(serde_json::Value::Array(serialized)))
        }
        StaticValue::Invalid => Err(()),
        StaticValue::Mapping
        | StaticValue::MatrixMapping(_)
        | StaticValue::NonStringable
        | StaticValue::Unknown => Ok(None),
    }
}

fn static_yaml_value(value: Value) -> StaticValue {
    match value {
        Value::Bool(value) => StaticValue::Bool(value),
        Value::Number(value) => StaticValue::Number(value.to_string()),
        Value::String(value) => StaticValue::String(value),
        Value::Null => StaticValue::Null,
        Value::Sequence(values) => {
            StaticValue::Sequence(values.into_iter().map(static_sequence_element).collect())
        }
        Value::Mapping(_) => StaticValue::Mapping,
        Value::Tagged(_) => StaticValue::Unknown,
    }
}

fn static_sequence_element(value: Value) -> StaticValue {
    match value {
        Value::Sequence(_) | Value::Mapping(_) | Value::Tagged(_) => StaticValue::NonStringable,
        value => static_yaml_value(value),
    }
}
