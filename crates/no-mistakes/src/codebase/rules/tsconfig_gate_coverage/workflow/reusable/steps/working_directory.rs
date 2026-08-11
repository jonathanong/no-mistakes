use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    EnvironmentState, InputState,
};
use crate::codebase::rules::tsconfig_gate_coverage::{command_scan, workflow::conditions};
use serde_yaml::Value;

pub(super) fn step_working_directory(
    step: &Value,
    inputs: &InputState,
    environment: &EnvironmentState,
    job_cwd: &Option<String>,
) -> Option<String> {
    step.get("working-directory")
        .and_then(Value::as_str)
        .map(|raw| {
            conditions::resolve_static_interpolations(raw, inputs, environment)
                .and_then(|directory| command_scan::normalize_repo_relative(&directory))
        })
        .unwrap_or_else(|| job_cwd.clone())
}
