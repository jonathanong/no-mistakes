use super::{InputState, StaticValue};
use serde_yaml::Value;

pub(super) fn input_name(operand: &str) -> Option<&str> {
    context_property_name(operand, "inputs")
}

pub(super) fn matrix_name(operand: &str) -> Option<&str> {
    context_property_name(operand, "matrix")
}

pub(super) fn github_event_name(operand: &str) -> bool {
    github_event_property(operand, "event_name")
}

pub(super) fn github_event_action(operand: &str) -> bool {
    context_property_name(operand, "github.event")
        .is_some_and(|property| property.eq_ignore_ascii_case("action"))
}

pub(super) fn matrix_property_value(name: &str, inputs: &InputState) -> StaticValue {
    inputs
        .get(&format!(
            "{}{}",
            super::inputs::MATRIX_VALUE_PREFIX,
            name.to_lowercase()
        ))
        .cloned()
        // GitHub resolves a missing matrix property to the empty string.
        .unwrap_or_else(|| {
            if super::inputs::matrix_property_is_dynamic(inputs) {
                StaticValue::Unknown
            } else {
                StaticValue::String(String::new())
            }
        })
}

pub(super) fn literal_from_json_static_value(expression: &str) -> Option<StaticValue> {
    static_scalar_from_json(
        crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::literal_from_json_value(expression.trim())?,
    )
}

pub(super) fn literal_from_json_sequence(expression: &str) -> Option<Vec<StaticValue>> {
    let Value::Sequence(values) = crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::literal_from_json_value(expression.trim())? else {
        return None;
    };
    values.into_iter().map(static_scalar_from_json).collect()
}

fn static_scalar_from_json(value: Value) -> Option<StaticValue> {
    match value {
        Value::Bool(value) => Some(StaticValue::Bool(value)),
        Value::Number(value) => Some(StaticValue::Number(value.to_string())),
        Value::String(value) => Some(StaticValue::String(value)),
        Value::Null => Some(StaticValue::Null),
        // Nested structured values would need their own coercion semantics.
        // Keep the analyzer conservative even inside literal membership.
        Value::Sequence(_) | Value::Mapping(_) | Value::Tagged(_) => None,
    }
}

pub(super) fn condition_input_value(operand: &str, inputs: &InputState) -> Option<StaticValue> {
    if let Some(name) = input_name(operand) {
        return Some(
            inputs
                .get(&name.to_lowercase())
                .cloned()
                .unwrap_or(StaticValue::Bool(false)),
        );
    }
    let name = matrix_name(operand)?;
    Some(matrix_property_value(name, inputs))
}

fn context_property_name<'a>(operand: &'a str, context: &str) -> Option<&'a str> {
    let operand = operand.trim();
    let remainder = operand
        .get(..context.len())
        .filter(|prefix| prefix.eq_ignore_ascii_case(context))
        .and_then(|_| operand.get(context.len()..))?;
    if let Some(name) = remainder.trim_start().strip_prefix('.') {
        let name = name.trim();
        return super::contracts::valid_identifier(name).then_some(name);
    }
    let bracketed = remainder.trim_start().strip_prefix('[')?.trim_start();
    let quote = bracketed.chars().next()?;
    if quote != '\'' {
        return None;
    }
    let name = bracketed.strip_prefix(quote)?;
    let (name, suffix) = name.split_once(quote)?;
    (suffix.trim() == "]" && super::contracts::valid_identifier(name)).then_some(name)
}

fn github_event_property(operand: &str, property: &str) -> bool {
    context_property_name(operand, "github")
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(property))
}
