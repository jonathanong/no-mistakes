use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    step_continue_on_error_value_valid, step_timeout_minutes_validity, EnvironmentState,
    InputState, StaticBool,
};
use serde_yaml::Value;

pub(super) fn step_configuration_validity(
    step: &Value,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> StaticBool {
    if !step_continue_on_error_value_valid(step, inputs, environment) {
        return StaticBool::False;
    }
    step_timeout_minutes_validity(step.get("timeout-minutes"), inputs, environment)
}
