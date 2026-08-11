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
