use crate::codebase::rules::tsconfig_gate_coverage::workflow::{
    conditions::{job_timeout_minutes_validity, EnvironmentState, InputState, StaticBool},
    reusable::validation::{
        environment_configuration_valid_for_inputs, job_concurrency_valid_for_inputs,
    },
    runtime::runs_on_has_statically_invalid_value,
};
use serde_yaml::Value;

pub(super) fn job_configuration_validity(
    job: &Value,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> StaticBool {
    if environment.has_invalid_value()
        || runs_on_has_statically_invalid_value(job, inputs)
        || !job_concurrency_valid_for_inputs(job.get("concurrency"), inputs)
        || !environment_configuration_valid_for_inputs(job, inputs, environment)
    {
        return StaticBool::False;
    }
    job_timeout_minutes_validity(job.get("timeout-minutes"), inputs)
}
