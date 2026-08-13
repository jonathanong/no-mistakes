use super::{expression_bool_with_status_and_environment, ConditionStatus, EnvironmentState};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    InputState, StaticBool,
};
use serde_yaml::Value;

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

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn continues_after_indeterminate_need(
    job: &Value,
    inputs: &InputState,
) -> bool {
    job.get("if")
        .and_then(Value::as_str)
        .is_some_and(|expression| {
            super::super::super::expressions::condition_has_status_function(expression)
                && [ConditionStatus::SUCCESS, ConditionStatus::FAILURE]
                    .into_iter()
                    .all(|status| {
                        expression_bool_with_status_and_environment(
                            expression,
                            inputs,
                            &EnvironmentState::default(),
                            status,
                        ) == StaticBool::True
                    })
        })
}

fn continues_after_unsuccessful_need(
    job: &Value,
    inputs: &InputState,
    status: ConditionStatus,
) -> bool {
    job.get("if")
        .and_then(Value::as_str)
        .is_some_and(|expression| {
            super::super::super::expressions::condition_has_status_function(expression)
                && expression_bool_with_status_and_environment(
                    expression,
                    inputs,
                    &EnvironmentState::default(),
                    status,
                ) == StaticBool::True
        })
}
