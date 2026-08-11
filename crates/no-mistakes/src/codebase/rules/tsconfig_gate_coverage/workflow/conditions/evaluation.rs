use super::{
    condition_values::{comparison_bool, condition_value},
    literals::{
        hexadecimal_bool, number_bool, quoted_string_bool, status_function_bool, strip_expression,
    },
    resolution::condition_input_value,
    ConditionStatus, EnvironmentState, InputState, StaticBool, StaticValue,
};
use serde_yaml::Value;

mod jobs;
pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) use jobs::{
    job_statically_disabled, job_statically_enabled, job_statically_enforcing,
    job_statically_not_enforcing,
};

/// Credit a timed step only when its timeout is statically known to be within
/// GitHub's 1..=360 minute step limit. Unknown (including dynamic matrices)
/// remains conservative rather than assuming a valid timeout.
pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn step_timeout_minutes_enforced(
    value: Option<&Value>,
    inputs: &InputState,
) -> bool {
    timeout_minutes_enforced(value, inputs, Some(360))
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn job_timeout_minutes_enforced(
    value: Option<&Value>,
    inputs: &InputState,
) -> bool {
    timeout_minutes_enforced(value, inputs, None)
}

fn timeout_minutes_enforced(
    value: Option<&Value>,
    inputs: &InputState,
    maximum: Option<u64>,
) -> bool {
    value.is_none_or(|value| match value {
        Value::Number(value) => value
            .as_u64()
            .is_some_and(|minutes| valid_timeout_minutes(minutes, maximum)),
        Value::String(expression) => {
            super::super::expressions::complete_literal_expression_value(expression)
                .or_else(|| {
                    let expression = strip_expression(expression.trim());
                    condition_input_value(expression, inputs, &EnvironmentState::default())
                        .and_then(|value| match value {
                            StaticValue::Number(value) => serde_yaml::from_str(&value).ok(),
                            _ => None,
                        })
                })
                .and_then(|value| value.as_u64())
                .is_some_and(|minutes| valid_timeout_minutes(minutes, maximum))
        }
        _ => false,
    })
}

fn valid_timeout_minutes(minutes: u64, maximum: Option<u64>) -> bool {
    minutes > 0 && maximum.is_none_or(|maximum| minutes <= maximum)
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn static_bool(
    value: Option<&Value>,
    inputs: &InputState,
) -> StaticBool {
    static_bool_with_environment(value, inputs, &EnvironmentState::default())
}

pub(super) fn static_bool_with_environment(
    value: Option<&Value>,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> StaticBool {
    static_bool_with_status_and_environment(value, inputs, environment, ConditionStatus::SUCCESS)
}

pub(super) fn static_bool_with_status_and_environment(
    value: Option<&Value>,
    inputs: &InputState,
    environment: &EnvironmentState,
    status: ConditionStatus,
) -> StaticBool {
    match value {
        Some(Value::Bool(value)) => StaticBool::from(*value),
        Some(Value::Number(value)) => number_bool(value.as_f64()),
        Some(Value::Null) => StaticBool::False,
        Some(Value::String(expression)) => {
            expression_bool_with_status_and_environment(expression, inputs, environment, status)
        }
        _ => StaticBool::Unknown,
    }
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn expression_bool(
    expression: &str,
    inputs: &InputState,
) -> StaticBool {
    expression_bool_with_status_and_environment(
        expression,
        inputs,
        &EnvironmentState::default(),
        ConditionStatus::SUCCESS,
    )
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn expression_bool_with_status_and_environment(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    status: ConditionStatus,
) -> StaticBool {
    let expression = strip_expression(expression.trim());
    if expression.is_empty() {
        return StaticBool::False;
    }
    if !super::super::expressions::condition_expression_valid(expression) {
        return StaticBool::Unknown;
    }
    if let Some(value) = super::logical::compound_bool(expression, inputs, environment, status) {
        return value;
    }
    if expression.eq_ignore_ascii_case("false") {
        return StaticBool::False;
    }
    if expression.eq_ignore_ascii_case("true") {
        return StaticBool::True;
    }
    if expression.eq_ignore_ascii_case("null") {
        return StaticBool::False;
    }
    if let Some(value) = status_function_bool(expression, status) {
        return value;
    }
    if let Some(value) =
        super::functions::static_function_bool(expression, inputs, environment, status)
    {
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
    resolve_input_expression(expression, inputs, environment, status)
}

fn resolve_input_expression(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    status: ConditionStatus,
) -> StaticBool {
    if let Some(value) = comparison_bool(expression, inputs, environment, status) {
        return value;
    }
    if let Some(value) = condition_value(expression, inputs, environment, status) {
        return value.truthiness();
    }
    if let Some(operand) = expression.strip_prefix('!').map(str::trim) {
        return expression_bool_with_status_and_environment(operand, inputs, environment, status)
            .negate();
    }
    StaticBool::Unknown
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn continues_after_skipped_need(
    job: &Value,
    inputs: &InputState,
) -> bool {
    continues_after_unsuccessful_need(job, inputs, ConditionStatus::SKIPPED)
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn continues_after_failed_need(
    job: &Value,
    inputs: &InputState,
) -> bool {
    continues_after_unsuccessful_need(job, inputs, ConditionStatus::FAILURE)
}

fn continues_after_unsuccessful_need(
    job: &Value,
    inputs: &InputState,
    status: ConditionStatus,
) -> bool {
    job.get("if")
        .and_then(Value::as_str)
        .is_some_and(|expression| {
            super::super::expressions::condition_has_status_function(expression)
                && expression_bool_with_status_and_environment(
                    expression,
                    inputs,
                    &EnvironmentState::default(),
                    status,
                ) == StaticBool::True
        })
}
