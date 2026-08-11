use super::{
    condition_values::{comparison_bool, condition_value},
    literals::{
        hexadecimal_bool, number_bool, quoted_string_bool, status_function_bool, strip_expression,
    },
    ConditionStatus, EnvironmentState, InputState, StaticBool, StaticValue,
};
use serde_yaml::Value;

mod jobs;
mod needs;
pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) use jobs::{
    job_statically_disabled, job_statically_enabled, job_statically_enforcing,
    job_statically_not_enforcing,
};
pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) use needs::{
    continues_after_failed_need, continues_after_indeterminate_need, continues_after_skipped_need,
};

/// Credit a timed step only when its timeout is statically known to be within
/// GitHub's 1..=360 minute step limit. Unknown (including dynamic matrices)
/// remains conservative rather than assuming a valid timeout.
pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn step_timeout_minutes_validity(
    value: Option<&Value>,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> StaticBool {
    timeout_minutes_validity(value, inputs, environment, Some(360))
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn job_timeout_minutes_validity(
    value: Option<&Value>,
    inputs: &InputState,
) -> StaticBool {
    timeout_minutes_validity(value, inputs, &EnvironmentState::default(), None)
}

fn timeout_minutes_validity(
    value: Option<&Value>,
    inputs: &InputState,
    environment: &EnvironmentState,
    maximum: Option<u64>,
) -> StaticBool {
    let Some(value) = value else {
        return StaticBool::True;
    };
    match value {
        Value::Number(value) => StaticBool::from(
            value
                .as_u64()
                .is_some_and(|minutes| valid_timeout_minutes(minutes, maximum)),
        ),
        Value::String(expression) => {
            if let Some(value) =
                super::super::expressions::complete_literal_expression_value(expression)
            {
                return yaml_timeout_validity(&value, maximum);
            }
            match super::complete_expression_static_value_with_environment(
                expression,
                inputs,
                environment,
            ) {
                Some(StaticValue::Number(value)) => serde_yaml::from_str(&value)
                    .ok()
                    .map_or(StaticBool::False, |value| {
                        yaml_timeout_validity(&value, maximum)
                    }),
                Some(StaticValue::Unknown) | None => StaticBool::Unknown,
                Some(_) => StaticBool::False,
            }
        }
        _ => StaticBool::False,
    }
}

fn yaml_timeout_validity(value: &Value, maximum: Option<u64>) -> StaticBool {
    StaticBool::from(
        value
            .as_u64()
            .is_some_and(|minutes| valid_timeout_minutes(minutes, maximum)),
    )
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
