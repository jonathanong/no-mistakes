use super::super::workflow::{
    concurrency_shape_valid, defaults_shape_valid, permissions_shape_valid,
};
use super::bindings::call_bindings_mapping_shape_valid;
use super::fields::{
    bool_or_expression_field_valid, condition_field_valid, strategy_shape_valid,
    string_field_valid, JOB_CONDITION_CONTEXTS,
};
use super::values::{
    container_shape_valid, environment_shape_valid, only_keys, outputs_shape_valid,
    runs_on_shape_valid, scalar_mapping_valid, services_shape_valid,
};
use serde_yaml::Value;

use crate::codebase::rules::tsconfig_gate_coverage::workflow::runtime::{
    container_runner_support, ContainerRunnerSupport,
};

const STEP_JOB_KEYS: &[&str] = &[
    "name",
    "permissions",
    "needs",
    "if",
    "runs-on",
    "environment",
    "concurrency",
    "outputs",
    "env",
    "defaults",
    "steps",
    "timeout-minutes",
    "continue-on-error",
    "container",
    "services",
    "strategy",
];

const REUSABLE_CALL_JOB_KEYS: &[&str] = &[
    "name",
    "uses",
    "with",
    "secrets",
    "strategy",
    "needs",
    "if",
    "concurrency",
    "permissions",
];

pub(crate) fn step_job_shape_valid(job: &Value) -> bool {
    job.as_mapping().is_some_and(|job| {
        only_keys(job, STEP_JOB_KEYS)
            && job.get("runs-on").is_some()
            && runs_on_shape_valid(job.get("runs-on"))
            && string_field_valid(job, "name")
            && condition_field_valid(job.get("if"), JOB_CONDITION_CONTEXTS, false)
            && permissions_shape_valid(job.get("permissions"))
            && environment_shape_valid(job.get("environment"))
            && concurrency_shape_valid(job.get("concurrency"))
            && outputs_shape_valid(job.get("outputs"))
            && scalar_mapping_valid(job.get("env"))
            && defaults_shape_valid(job.get("defaults"))
            && super::fields::number_or_expression_field_valid(job, "timeout-minutes")
            && bool_or_expression_field_valid(job, "continue-on-error")
            && container_shape_valid(job.get("container"))
            && services_shape_valid(job.get("services"))
            && ((job.get("container").is_none() && job.get("services").is_none())
                || container_runner_support(job) != ContainerRunnerSupport::NonLinux)
            && strategy_shape_valid(job.get("strategy"))
    })
}

pub(crate) fn reusable_call_job_shape_valid(job: &Value) -> bool {
    job.as_mapping().is_some_and(|job| {
        only_keys(job, REUSABLE_CALL_JOB_KEYS)
            && job
                .get("uses")
                .and_then(Value::as_str)
                .is_some_and(|uses| !uses.is_empty())
            && string_field_valid(job, "name")
            && condition_field_valid(job.get("if"), JOB_CONDITION_CONTEXTS, false)
            && permissions_shape_valid(job.get("permissions"))
            && concurrency_shape_valid(job.get("concurrency"))
            && strategy_shape_valid(job.get("strategy"))
            && call_bindings_mapping_shape_valid(job)
    })
}
