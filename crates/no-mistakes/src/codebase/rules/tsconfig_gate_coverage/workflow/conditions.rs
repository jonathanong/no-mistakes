use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

mod contracts;
mod input_value;
mod inputs;
mod literals;
mod logical;

use input_value::{comparison_literal, input_name};
pub(super) use inputs::{
    callee_inputs, callee_secrets_valid, direct_inputs, inputs_with_matrix_values,
};
use literals::{
    hexadecimal_bool, number_bool, quoted_string_bool, status_function_bool, strip_expression,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum StaticBool {
    False,
    True,
    TruthyNonBoolean,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum StaticValue {
    Bool(bool),
    String(String),
    Number(String),
    Null,
    Unknown,
}

pub(super) type InputState = BTreeMap<String, StaticValue>;
const EVENT_NAME_KEY: &str = "\0github.event_name";

fn event_name_value(inputs: &InputState) -> Option<StaticValue> {
    inputs.get(EVENT_NAME_KEY).cloned()
}

pub(super) fn statically_skipped_jobs(
    jobs: &serde_yaml::Mapping,
    inputs: &InputState,
    initial_skipped: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut skipped = initial_skipped.clone();
    loop {
        let mut changed = false;
        for (job_id, job) in jobs {
            let job_id = super::normalized_job_id(job_id).expect("validated scalar job ID");
            let directly_disabled = static_bool(job.get("if"), inputs) == StaticBool::False;
            let blocked_by_need = !continues_after_skipped_need(job, inputs)
                && crate::codebase::workflow_topology::value_primitives::string_list(
                    job.get("needs"),
                )
                .iter()
                .any(|need| skipped.contains(&need.to_lowercase()));
            if (directly_disabled || blocked_by_need) && skipped.insert(job_id) {
                changed = true;
            }
        }
        if !changed {
            return skipped;
        }
    }
}

pub(super) fn statically_not_enforcing(value: &Value, inputs: &InputState) -> bool {
    static_bool(value.get("if"), inputs) == StaticBool::False
        || static_bool(value.get("continue-on-error"), inputs) == StaticBool::True
}

fn static_bool(value: Option<&Value>, inputs: &InputState) -> StaticBool {
    match value {
        Some(Value::Bool(value)) => StaticBool::from(*value),
        Some(Value::Number(value)) => number_bool(value.as_f64()),
        Some(Value::Null) => StaticBool::False,
        Some(Value::String(expression)) => expression_bool(expression, inputs),
        _ => StaticBool::Unknown,
    }
}

fn expression_bool(expression: &str, inputs: &InputState) -> StaticBool {
    expression_bool_with_status(expression, inputs, StaticBool::True)
}

fn expression_bool_with_status(
    expression: &str,
    inputs: &InputState,
    success: StaticBool,
) -> StaticBool {
    let expression = strip_expression(expression.trim());
    if super::expressions::condition_expression_valid(expression) {
        if let Some(value) = logical::compound_bool(expression, inputs, success) {
            return value;
        }
    }
    if expression.is_empty() || expression.eq_ignore_ascii_case("false") {
        return StaticBool::False;
    }
    if expression.eq_ignore_ascii_case("true") {
        return StaticBool::True;
    }
    if expression.eq_ignore_ascii_case("null") {
        return StaticBool::False;
    }
    if let Some(value) = status_function_bool(expression, success) {
        return value;
    }
    if let Some(value) = quoted_string_bool(expression) {
        return value;
    }
    if let Some(value) = hexadecimal_bool(expression) {
        return value;
    }
    if let Ok(value) = expression.parse::<f64>() {
        return number_bool(Some(value));
    }
    resolve_input_expression(expression, inputs, success)
}

fn resolve_input_expression(
    expression: &str,
    inputs: &InputState,
    success: StaticBool,
) -> StaticBool {
    if let Some((left, right, equal)) = logical::comparison_operands(expression) {
        let (actual, expected) = match (
            condition_value(left, inputs, success),
            comparison_literal(right),
        ) {
            (Some(actual), Some(expected)) => (actual, expected),
            _ => match (
                comparison_literal(left),
                condition_value(right, inputs, success),
            ) {
                (Some(expected), Some(actual)) => (actual, expected),
                _ => return StaticBool::Unknown,
            },
        };
        let value = actual.equals(&expected);
        return if equal { value } else { value.negate() };
    }
    if let Some(name) = input_name(expression) {
        return inputs
            .get(&name.to_lowercase())
            .cloned()
            .unwrap_or(StaticValue::Bool(false))
            .truthiness();
    }
    if let Some(name) = expression
        .strip_prefix('!')
        .map(str::trim)
        .and_then(input_name)
    {
        return inputs
            .get(&name.to_lowercase())
            .cloned()
            .unwrap_or(StaticValue::Bool(false))
            .truthiness()
            .negate();
    }
    StaticBool::Unknown
}

fn condition_value(operand: &str, inputs: &InputState, success: StaticBool) -> Option<StaticValue> {
    if let Some(value) = status_function_bool(operand.trim(), success) {
        return match value {
            StaticBool::False => Some(StaticValue::Bool(false)),
            StaticBool::True => Some(StaticValue::Bool(true)),
            StaticBool::TruthyNonBoolean | StaticBool::Unknown => None,
        };
    }
    if operand.trim().eq_ignore_ascii_case("github.event_name") {
        return event_name_value(inputs);
    }
    let name = input_name(operand)?;
    Some(
        inputs
            .get(&name.to_lowercase())
            .cloned()
            .unwrap_or(StaticValue::Bool(false)),
    )
}

fn continues_after_skipped_need(job: &Value, inputs: &InputState) -> bool {
    job.get("if")
        .and_then(Value::as_str)
        .is_some_and(|expression| {
            super::expressions::condition_has_status_function(expression)
                && expression_bool_with_status(expression, inputs, StaticBool::False)
                    == StaticBool::True
        })
}

#[cfg(test)]
mod tests;
