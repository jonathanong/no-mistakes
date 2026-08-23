use super::{
    evaluation::{static_bool_with_environment, static_bool_with_status_and_environment},
    ConditionStatus, EnvironmentState, InputState, StaticBool,
};
use serde_yaml::Value;

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn step_condition_with_status(
    value: &Value,
    inputs: &InputState,
    environment: &EnvironmentState,
    success: StaticBool,
) -> StaticBool {
    let condition = match value.get("if") {
        Some(condition) => static_bool_with_status_and_environment(
            Some(condition),
            inputs,
            environment,
            ConditionStatus::from_success(success),
        ),
        None => return success,
    };
    let has_status_function = value
        .get("if")
        .and_then(Value::as_str)
        .is_some_and(super::super::expressions::condition_has_status_function);
    if has_status_function {
        condition
    } else {
        implicit_success_condition(success, condition)
    }
}

fn implicit_success_condition(success: StaticBool, condition: StaticBool) -> StaticBool {
    match (success.truthiness(), condition.truthiness()) {
        (StaticBool::False, _) | (_, StaticBool::False) => StaticBool::False,
        (StaticBool::True, StaticBool::True) => StaticBool::True,
        _ => StaticBool::Unknown,
    }
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn continue_on_error_value(
    value: &Value,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> StaticBool {
    value
        .get("continue-on-error")
        .map(|value| static_bool_with_environment(Some(value), inputs, environment))
        .unwrap_or(StaticBool::False)
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn step_continue_on_error_value_valid(
    value: &Value,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> bool {
    let Some(value) = value.get("continue-on-error") else {
        return true;
    };
    match value {
        Value::Bool(_) => true,
        Value::String(expression) => {
            match super::complete_expression_static_value_with_environment(
                expression,
                inputs,
                environment,
            ) {
                Some(super::StaticValue::Bool(_)) | None => true,
                Some(_) => false,
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;
