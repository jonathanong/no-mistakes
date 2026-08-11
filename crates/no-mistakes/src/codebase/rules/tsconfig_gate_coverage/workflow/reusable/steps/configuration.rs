use crate::codebase::rules::tsconfig_gate_coverage::{
    command_scan,
    workflow::{
        conditions::{
            resolve_static_interpolations, step_continue_on_error_value_valid,
            step_timeout_minutes_validity, EnvironmentState, InputState, StaticBool,
        },
        default_working_directory,
        runtime::{effective_shell, runs_on_can_default_to_windows},
    },
};
use serde_yaml::Value;

pub(super) fn job_working_directory(
    job: &Value,
    inputs: &InputState,
    environment: &EnvironmentState,
    workflow_cwd: Option<String>,
) -> Option<String> {
    let Some(raw) = default_working_directory(job) else {
        return workflow_cwd;
    };
    resolve_static_interpolations(raw, inputs, environment)
        .and_then(|directory| command_scan::normalize_repo_relative(&directory))
}

pub(super) fn job_runtime(
    job: &Value,
    inputs: &InputState,
    workflow_shell: Option<String>,
) -> (Option<String>, bool) {
    (
        effective_shell(job, workflow_shell),
        runs_on_can_default_to_windows(job, inputs),
    )
}

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
