use super::{EnvironmentState, InputState, StaticValue};
use serde_yaml::Value;

pub(super) fn input_name(operand: &str) -> Option<&str> {
    context_property_name(operand, "inputs")
}

pub(super) fn matrix_name(operand: &str) -> Option<&str> {
    context_property_name(operand, "matrix")
}

pub(super) fn env_name(operand: &str) -> Option<&str> {
    context_property_name(operand, "env")
}

fn needs_result_name(operand: &str) -> Option<&str> {
    let operand = operand.trim();
    let remainder = operand
        .get(.."needs".len())
        .filter(|prefix| prefix.eq_ignore_ascii_case("needs"))
        .and_then(|_| operand.get("needs".len()..))?;
    let (name, remainder) = context_property_segment(remainder)?;
    let remainder = github_property_segment(remainder, "result")?;
    remainder.trim().is_empty().then_some(name)
}

fn step_outcome_name(operand: &str) -> Option<&str> {
    let operand = operand.trim();
    let remainder = operand
        .get(.."steps".len())
        .filter(|prefix| prefix.eq_ignore_ascii_case("steps"))
        .and_then(|_| operand.get("steps".len()..))?;
    let (name, remainder) = context_property_segment(remainder)?;
    let remainder = github_property_segment(remainder, "outcome")?;
    remainder.trim().is_empty().then_some(name)
}

pub(super) fn needs_result_is_known_not_skipped(operand: &str, inputs: &InputState) -> bool {
    needs_result_name(operand)
        .is_some_and(|name| super::inputs::needs_result_not_skipped(name, inputs))
}

fn context_property_segment(remainder: &str) -> Option<(&str, &str)> {
    let remainder = remainder.trim_start();
    if let Some(remainder) = remainder.strip_prefix('.') {
        let remainder = remainder.trim_start();
        let end = remainder
            .find(|character: char| {
                character == '.' || character == '[' || character.is_whitespace()
            })
            .unwrap_or(remainder.len());
        let name = &remainder[..end];
        return super::contracts::valid_identifier(name).then_some((name, &remainder[end..]));
    }
    let quoted = remainder
        .strip_prefix('[')?
        .trim_start()
        .strip_prefix('\'')?;
    let (name, remainder) = quoted.split_once('\'')?;
    let remainder = remainder.trim_start().strip_prefix(']')?;
    super::contracts::valid_identifier(name).then_some((name, remainder))
}

pub(super) fn secret_name(operand: &str) -> Option<&str> {
    let body = operand
        .trim()
        .strip_prefix("${{")?
        .strip_suffix("}}")?
        .trim();
    context_property_name(body, "secrets")
}

pub(super) fn github_event_name(operand: &str) -> bool {
    github_event_property(operand, &["event_name"])
}

pub(super) fn github_event_action(operand: &str) -> bool {
    github_event_property(operand, &["event", "action"])
}

pub(super) fn github_ref(operand: &str) -> bool {
    github_event_property(operand, &["ref"])
}

pub(super) fn github_ref_name(operand: &str) -> bool {
    github_event_property(operand, &["ref_name"])
}

pub(super) fn github_base_ref(operand: &str) -> bool {
    github_event_property(operand, &["base_ref"])
}

fn github_event_property(operand: &str, properties: &[&str]) -> bool {
    let operand = operand.trim();
    let Some(remainder) = operand
        .get(.."github".len())
        .filter(|prefix| prefix.eq_ignore_ascii_case("github"))
        .and_then(|_| operand.get("github".len()..))
    else {
        return false;
    };
    let Some(remainder) = properties
        .iter()
        .try_fold(remainder, |remainder, property| {
            github_property_segment(remainder, property)
        })
    else {
        return false;
    };
    remainder.trim().is_empty()
}

fn github_property_segment<'a>(remainder: &'a str, expected: &str) -> Option<&'a str> {
    let remainder = remainder.trim_start();
    if let Some(remainder) = remainder.strip_prefix('.') {
        let remainder = remainder.trim_start();
        let property = remainder.get(..expected.len())?;
        return property
            .eq_ignore_ascii_case(expected)
            .then_some(&remainder[expected.len()..]);
    }
    let remainder = remainder.strip_prefix('[')?.trim_start();
    let quoted = remainder.strip_prefix('\'')?;
    let (property, remainder) = quoted.split_once('\'')?;
    property
        .eq_ignore_ascii_case(expected)
        .then_some(remainder.trim_start().strip_prefix(']')?)
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
    Some(static_yaml_value(
        crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::literal_from_json_value(expression.trim())?,
    ))
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
        Value::Mapping(_) | Value::Tagged(_) => StaticValue::Unknown,
    }
}

fn static_sequence_element(value: Value) -> StaticValue {
    match value {
        Value::Sequence(_) | Value::Mapping(_) | Value::Tagged(_) => StaticValue::NonStringable,
        value => static_yaml_value(value),
    }
}

pub(super) fn condition_input_value(
    operand: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> Option<StaticValue> {
    if let Some(name) = input_name(operand) {
        return Some(
            inputs
                .get(&name.to_lowercase())
                .cloned()
                .unwrap_or(StaticValue::Bool(false)),
        );
    }
    if let Some(name) = env_name(operand) {
        return Some(
            environment
                .value(name)
                .unwrap_or(StaticValue::String(String::new())),
        );
    }
    if let Some(name) = needs_result_name(operand) {
        return Some(super::inputs::needs_result_value(name, inputs));
    }
    if let Some(name) = step_outcome_name(operand) {
        return Some(environment.step_outcome(name));
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
