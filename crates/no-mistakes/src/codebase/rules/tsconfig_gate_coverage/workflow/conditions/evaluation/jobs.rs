use super::super::{ConditionStatus, EnvironmentState, InputState, StaticBool};
use super::{static_bool, static_bool_with_status_and_environment};
use serde_yaml::Value;

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn job_statically_not_enforcing(
    value: &Value,
    inputs: &InputState,
) -> bool {
    job_enforcement(value, inputs, false).0
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn job_statically_disabled(
    value: &Value,
    inputs: &InputState,
) -> bool {
    value
        .get("if")
        .is_some_and(|condition| static_bool(Some(condition), inputs) == StaticBool::False)
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn job_statically_enabled(
    value: &Value,
    inputs: &InputState,
) -> bool {
    value
        .get("if")
        .is_none_or(|condition| static_bool(Some(condition), inputs) == StaticBool::True)
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn job_statically_enforcing(
    value: &Value,
    inputs: &InputState,
    after_failed_need: bool,
) -> bool {
    job_enforcement(value, inputs, after_failed_need).1
}

fn job_enforcement(value: &Value, inputs: &InputState, after_failed_need: bool) -> (bool, bool) {
    let status = if after_failed_need {
        ConditionStatus::FAILURE
    } else {
        ConditionStatus::SUCCESS
    };
    let condition = value.get("if").map_or(StaticBool::True, |condition| {
        static_bool_with_status_and_environment(
            Some(condition),
            inputs,
            &EnvironmentState::default(),
            status,
        )
    });
    let continue_on_error =
        value
            .get("continue-on-error")
            .map_or(StaticBool::False, |continue_on_error| {
                static_bool_with_status_and_environment(
                    Some(continue_on_error),
                    inputs,
                    &EnvironmentState::default(),
                    status,
                )
            });
    (
        matches!(condition, StaticBool::False | StaticBool::Invalid)
            || matches!(continue_on_error, StaticBool::True | StaticBool::Invalid),
        condition == StaticBool::True && continue_on_error == StaticBool::False,
    )
}
