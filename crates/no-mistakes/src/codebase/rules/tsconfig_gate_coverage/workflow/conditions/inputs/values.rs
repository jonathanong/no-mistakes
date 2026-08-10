use super::{JsonScalar, Value, WorkflowCallInputType};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    input_value::comparison_literal, InputState, StaticValue,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    complete_expression_type, StaticExpressionType,
};

pub(super) fn default_value(
    default: Option<&JsonScalar>,
    input_type: WorkflowCallInputType,
) -> StaticValue {
    match (input_type, default) {
        (WorkflowCallInputType::String, Some(JsonScalar::Text(value))) => {
            StaticValue::String(value.clone())
        }
        (WorkflowCallInputType::Number, Some(JsonScalar::Number(value))) => {
            StaticValue::Number(value.to_string())
        }
        (WorkflowCallInputType::String, None) => StaticValue::String(String::new()),
        (WorkflowCallInputType::Number, None) => StaticValue::Number("0".to_string()),
        _ => StaticValue::Unknown,
    }
}

pub(super) fn nonboolean_binding_value(
    value: &Value,
    parent: &InputState,
    input_type: WorkflowCallInputType,
) -> StaticValue {
    if let Some(value) = forwarded_input_value(value, parent) {
        return value;
    }
    if let Some(value) = value.as_str().and_then(static_expression_value) {
        return value;
    }
    if value.as_str().is_some_and(|value| value.contains("${{")) {
        return StaticValue::Unknown;
    }
    match (input_type, value) {
        (WorkflowCallInputType::String, Value::String(value)) => StaticValue::String(value.clone()),
        (WorkflowCallInputType::Number, Value::Number(value)) => {
            StaticValue::Number(value.to_string())
        }
        _ => StaticValue::Unknown,
    }
}

pub(super) fn forwarded_input_value(value: &Value, parent: &InputState) -> Option<StaticValue> {
    let body = value
        .as_str()?
        .trim()
        .strip_prefix("${{")?
        .strip_suffix("}}")?
        .trim();
    let name = body.strip_prefix("inputs.")?.trim();
    parent.get(&name.to_lowercase()).cloned()
}

fn static_expression_value(text: &str) -> Option<StaticValue> {
    let expression_type = complete_expression_type(text.trim())?;
    let body = text.trim().strip_prefix("${{")?.strip_suffix("}}")?.trim();
    match expression_type {
        StaticExpressionType::Dynamic => Some(StaticValue::Unknown),
        StaticExpressionType::Null
        | StaticExpressionType::Boolean
        | StaticExpressionType::String
        | StaticExpressionType::Number => comparison_literal(body),
    }
}
