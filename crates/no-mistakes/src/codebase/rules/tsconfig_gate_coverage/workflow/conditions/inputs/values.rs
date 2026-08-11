use super::{JsonScalar, Value, WorkflowCallInputType};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    event_action_value, event_name_value,
    input_value::comparison_literal,
    resolution::{
        github_event_action, github_event_name, input_name, matrix_name, matrix_property_value,
    },
    InputState, StaticBool, StaticValue,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    complete_expression_type, complete_literal_expression_value, StaticExpressionType,
};

pub(super) fn default_value(
    default: Option<&JsonScalar>,
    input_type: WorkflowCallInputType,
    activation_inputs: &InputState,
) -> StaticValue {
    if let Some(JsonScalar::Text(value)) = default {
        if let Some(value) = static_expression_value(value, activation_inputs) {
            return value;
        }
        if value.contains("${{") {
            return StaticValue::Unknown;
        }
    }
    match (input_type, default) {
        (WorkflowCallInputType::Boolean, Some(JsonScalar::Bool(value))) => {
            StaticValue::Bool(*value)
        }
        (WorkflowCallInputType::String, Some(JsonScalar::Text(value))) => {
            StaticValue::String(value.clone())
        }
        (WorkflowCallInputType::Number, Some(JsonScalar::Number(value))) => {
            StaticValue::Number(value.to_string())
        }
        (WorkflowCallInputType::String, None) => StaticValue::String(String::new()),
        (WorkflowCallInputType::Number, None) => StaticValue::Number("0".to_string()),
        (WorkflowCallInputType::Boolean, None) => StaticValue::Bool(false),
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
    if let Some(value) = value
        .as_str()
        .and_then(|value| static_expression_value(value, parent))
    {
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
    if github_event_name(body) {
        return event_name_value(parent);
    }
    if github_event_action(body) {
        return event_action_value(parent);
    }
    if let Some(name) = matrix_name(body) {
        return Some(matrix_property_value(name, parent));
    }
    let name = input_name(body)?;
    parent.get(&name.to_lowercase()).cloned()
}

pub(super) fn matrix_axis_value(value: &Value) -> Option<StaticValue> {
    match value {
        Value::Bool(value) => Some(StaticValue::Bool(*value)),
        Value::Number(value) => Some(StaticValue::Number(value.to_string())),
        Value::String(value) => static_expression_value(value, &InputState::new())
            .or_else(|| Some(StaticValue::String(value.clone()))),
        Value::Null => Some(StaticValue::Null),
        // Static object-valued matrix entries are known values even though
        // GitHub expressions cannot stringify them. Keep that distinct from
        // an omitted matrix property, which coerces to an empty string.
        Value::Mapping(_) => Some(StaticValue::NonStringable),
        Value::Sequence(_) | Value::Tagged(_) => None,
    }
}

fn static_expression_value(text: &str, activation_inputs: &InputState) -> Option<StaticValue> {
    let body = text.trim().strip_prefix("${{")?.strip_suffix("}}")?.trim();
    if let Some(value) = forwarded_input_value(&Value::String(text.to_string()), activation_inputs)
    {
        return Some(value);
    }
    if let Some(value) = comparison_literal(body) {
        return Some(value);
    }
    if let Some(value) = complete_literal_expression_value(text) {
        return Some(match value {
            Value::Bool(value) => StaticValue::Bool(value),
            Value::Number(value) => StaticValue::Number(value.to_string()),
            Value::String(value) => StaticValue::String(value),
            Value::Null => StaticValue::Null,
            Value::Sequence(_) | Value::Mapping(_) | Value::Tagged(_) => StaticValue::Unknown,
        });
    }
    let expression_type = complete_expression_type(text.trim())?;
    match expression_type {
        StaticExpressionType::Dynamic => Some(StaticValue::Unknown),
        StaticExpressionType::Boolean => {
            match super::super::expression_bool(text, activation_inputs) {
                StaticBool::False => Some(StaticValue::Bool(false)),
                StaticBool::True => Some(StaticValue::Bool(true)),
                StaticBool::TruthyNonBoolean | StaticBool::Unknown => Some(StaticValue::Unknown),
            }
        }
        StaticExpressionType::Null
        | StaticExpressionType::String
        | StaticExpressionType::Number => None,
    }
}

#[cfg(test)]
mod tests;
