use super::{EnvironmentState, InputState, StaticValue};

mod properties;
pub(crate) use properties::context_output_name;
use properties::{context_property_name, context_property_segment, github_property_segment};
pub(super) use properties::{
    github_base_ref, github_event_action, github_event_name, github_head_ref, github_ref,
    github_ref_name, github_ref_type,
};

pub(super) fn input_name(operand: &str) -> Option<&str> {
    context_property_name(operand, "inputs")
}

pub(super) fn matrix_property_path(operand: &str) -> Option<Vec<&str>> {
    let operand = operand.trim();
    let mut remainder = operand
        .get(.."matrix".len())
        .filter(|prefix| prefix.eq_ignore_ascii_case("matrix"))
        .and_then(|_| operand.get("matrix".len()..))?;
    let mut path = Vec::new();
    while !remainder.trim().is_empty() {
        let (name, next) = context_property_segment(remainder)?;
        path.push(name);
        remainder = next;
    }
    (!path.is_empty()).then_some(path)
}

fn strategy_name(operand: &str) -> Option<&str> {
    context_property_name(operand, "strategy")
}

pub(super) fn env_name(operand: &str) -> Option<&str> {
    context_property_name(operand, "env")
}

pub(super) fn runner_os(operand: &str) -> bool {
    context_property_name(operand, "runner").is_some_and(|name| name.eq_ignore_ascii_case("os"))
}

pub(super) fn job_status(operand: &str) -> bool {
    context_property_name(operand, "job").is_some_and(|name| name.eq_ignore_ascii_case("status"))
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

fn step_result_name(operand: &str) -> Option<(&str, StepResult)> {
    let operand = operand.trim();
    let remainder = operand
        .get(.."steps".len())
        .filter(|prefix| prefix.eq_ignore_ascii_case("steps"))
        .and_then(|_| operand.get("steps".len()..))?;
    let (name, remainder) = context_property_segment(remainder)?;
    let result = if let Some(remainder) = github_property_segment(remainder, "outcome") {
        (remainder, StepResult::Outcome)
    } else {
        (
            github_property_segment(remainder, "conclusion")?,
            StepResult::Conclusion,
        )
    };
    result.0.trim().is_empty().then_some((name, result.1))
}

enum StepResult {
    Outcome,
    Conclusion,
}

pub(super) fn needs_result_is_known_not_skipped(operand: &str, inputs: &InputState) -> bool {
    needs_result_name(operand)
        .is_some_and(|name| super::inputs::needs_result_not_skipped(name, inputs))
}

pub(super) fn secret_name(operand: &str) -> Option<&str> {
    let body = operand
        .trim()
        .strip_prefix("${{")?
        .strip_suffix("}}")?
        .trim();
    context_property_name(body, "secrets")
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

pub(super) fn matrix_property_path_value(path: &[&str], inputs: &InputState) -> StaticValue {
    let mut key = format!(
        "{}{}",
        super::inputs::MATRIX_VALUE_PREFIX,
        path[0].to_lowercase()
    );
    let Some(mut value) = inputs.get(&key).cloned() else {
        return matrix_property_value(path[0], inputs);
    };
    for name in &path[1..] {
        if !matches!(value, StaticValue::MatrixMapping(_)) {
            return StaticValue::Unknown;
        }
        key.push('.');
        key.push_str(&name.to_lowercase());
        value = inputs
            .get(&key)
            .cloned()
            // A missing property of a known object coerces to an empty string.
            .unwrap_or_else(|| StaticValue::String(String::new()));
    }
    value
}

fn strategy_property_value(name: &str, inputs: &InputState) -> StaticValue {
    inputs
        .get(&format!(
            "{}{}",
            super::inputs::STRATEGY_VALUE_PREFIX,
            name.to_lowercase()
        ))
        .cloned()
        .unwrap_or(StaticValue::Unknown)
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
    if let Some((job, output)) = context_output_name(operand, "needs") {
        return Some(super::inputs::needs_output_value(job, output, inputs));
    }
    if let Some((name, result)) = step_result_name(operand) {
        return Some(match result {
            StepResult::Outcome => environment.step_outcome(name),
            StepResult::Conclusion => environment.step_conclusion(name),
        });
    }
    if runner_os(operand) {
        return Some(environment.runner_os());
    }
    if let Some(name) = strategy_name(operand) {
        return Some(strategy_property_value(name, inputs));
    }
    let path = matrix_property_path(operand)?;
    Some(matrix_property_path_value(&path, inputs))
}
