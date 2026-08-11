use crate::codebase::rules::tsconfig_gate_coverage::workflow::{
    conditions::{job_timeout_minutes_validity, InputState, StaticBool},
    reusable::validation::{
        environment_configuration_valid_for_inputs, job_concurrency_valid_for_inputs,
    },
};
use serde_yaml::Value;

pub(super) fn job_configuration_validity(job: &Value, inputs: &InputState) -> StaticBool {
    if !job_concurrency_valid_for_inputs(job.get("concurrency"), inputs)
        || !environment_configuration_valid_for_inputs(job, inputs)
    {
        return StaticBool::False;
    }
    job_timeout_minutes_validity(job.get("timeout-minutes"), inputs)
}
