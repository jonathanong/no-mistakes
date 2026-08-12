use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    EnvironmentState, InputState,
};
use crate::codebase::rules::tsconfig_gate_coverage::{command_scan, workflow::conditions};
use serde_yaml::Value;
use std::collections::BTreeSet;

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

pub(super) fn working_directory_exists(directory: &str, visible_paths: &BTreeSet<String>) -> bool {
    directory == "."
        || visible_paths.iter().any(|path| {
            path == directory
                || path
                    .strip_prefix(directory)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        })
}

#[cfg(test)]
mod tests;
