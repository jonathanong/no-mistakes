use super::super::conditions::InputState;
use super::super::effective_working_directory;
use super::super::runtime::{
    container_runner_support, effective_shell, has_static_runnable_runs_on, ContainerRunnerSupport,
};
use super::model::{ActivationKey, ActivationMemo, ScanContext, WorkflowDocument};
use super::validation::{
    call_bindings_shape_valid, reusable_call_job_shape_valid, valid_job_dependencies,
    workflow_shape_valid,
};
use crate::codebase::ci_graph::triggers::CompiledTriggers;
use serde_yaml::Value;
use std::collections::BTreeSet;

mod jobs;
use jobs::{JobScanner, JobStates};

pub(super) fn scan_activation(
    path: &str,
    document: &WorkflowDocument<'_>,
    triggers: &CompiledTriggers,
    inputs: &InputState,
    active_paths: &BTreeSet<String>,
    context: &ScanContext<'_>,
    memo: &mut ActivationMemo,
) -> Option<BTreeSet<String>> {
    if active_paths.contains(path) {
        return None;
    }
    let key = ActivationKey {
        path: path.to_string(),
        inputs: inputs.clone(),
        active_paths: active_paths.clone(),
    };
    if let Some(result) = memo.get(&key) {
        return result.clone();
    }
    memo.record_computation();
    let result = scan_activation_uncached(
        path,
        document,
        triggers,
        inputs,
        active_paths,
        context,
        memo,
    );
    memo.insert(key, result.clone());
    result
}

fn scan_activation_uncached(
    path: &str,
    document: &WorkflowDocument<'_>,
    triggers: &CompiledTriggers,
    inputs: &InputState,
    active_paths: &BTreeSet<String>,
    context: &ScanContext<'_>,
    memo: &mut ActivationMemo,
) -> Option<BTreeSet<String>> {
    let mut active_paths = active_paths.clone();
    active_paths.insert(path.to_string());
    if !workflow_shape_valid(document.value) {
        return None;
    }
    let workflow_cwd = effective_working_directory(document.value, Some(".".to_string()));
    let workflow_shell = effective_shell(document.value, None);
    let jobs = document.value.get("jobs").and_then(Value::as_mapping)?;
    if jobs.is_empty() || !valid_job_dependencies(jobs) {
        return None;
    }
    let job_states = JobStates::new(jobs, inputs)?;
    JobScanner::new(
        &job_states,
        triggers,
        workflow_cwd,
        workflow_shell,
        &active_paths,
        context,
        memo,
    )
    .scan(jobs)
}

fn reusable_call_target(job: &Value) -> Option<Option<&str>> {
    match job.get("uses") {
        Some(Value::String(target))
            if reusable_call_job_shape_valid(job) && call_bindings_shape_valid(job) =>
        {
            Some(Some(target))
        }
        Some(_) => None,
        None => Some(None),
    }
}

fn step_job_runner_supported(job: &Value) -> bool {
    if !has_static_runnable_runs_on(job) {
        return false;
    }
    let requires_linux_runner = job.get("container").is_some() || job.get("services").is_some();
    !requires_linux_runner
        || job
            .as_mapping()
            .is_some_and(|job| container_runner_support(job) == ContainerRunnerSupport::Linux)
}
