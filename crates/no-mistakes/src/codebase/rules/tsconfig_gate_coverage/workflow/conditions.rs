use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

mod condition_values;
mod contracts;
mod functions;
mod input_value;
mod inputs;
mod literals;
mod logical;
mod resolution;

use condition_values::{comparison_bool, condition_value};
pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) use contracts::valid_identifier;
use input_value::comparison_literal;
use inputs::event_name_value;
pub(super) use inputs::{
    callee_inputs, callee_secrets, direct_inputs, inputs_with_matrix_values, MatrixState,
    SecretState,
};
use literals::{
    hexadecimal_bool, number_bool, quoted_string_bool, status_function_bool, strip_expression,
};
use resolution::{condition_input_value, input_name, literal_from_json_static_value};

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

pub(super) fn statically_skipped_jobs(
    jobs: &serde_yaml::Mapping,
    initial_skipped: &BTreeSet<String>,
    matrix_inputs: impl Fn(&Value, &Value) -> Vec<InputState>,
) -> BTreeSet<String> {
    let mut skipped = initial_skipped.clone();
    loop {
        let mut changed = false;
        for (raw_job_id, job) in jobs {
            let job_id = super::normalized_job_id(raw_job_id).expect("validated scalar job ID");
            let inputs = matrix_inputs(raw_job_id, job);
            let directly_disabled = !inputs.is_empty()
                && inputs
                    .iter()
                    .all(|inputs| static_bool(job.get("if"), inputs) == StaticBool::False);
            let blocked_by_need = !inputs
                .iter()
                .any(|inputs| continues_after_skipped_need(job, inputs))
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
    if let Some(value) = functions::static_function_bool(expression, inputs, success) {
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
    if let Some(value) = comparison_bool(expression, inputs, success) {
        return value;
    }
    if let Some(value) = literal_from_json_static_value(expression)
        .or_else(|| condition_input_value(expression, inputs))
    {
        return value.truthiness();
    }
    if let Some(operand) = expression.strip_prefix('!').map(str::trim) {
        return expression_bool_with_status(operand, inputs, success).negate();
    }
    StaticBool::Unknown
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
mod matrix_tests;
#[cfg(test)]
mod relational_tests;
#[cfg(test)]
mod tests;
