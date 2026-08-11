use super::super::effective_working_directory;
use super::super::runtime::{
    container_runner_support, effective_shell, has_static_runnable_runs_on, ContainerRunnerSupport,
};
use super::model::{ActivationKey, ActivationMemo, ActivationState, ScanContext, WorkflowDocument};
use super::validation::{
    call_bindings_shape_valid, reusable_call_job_shape_valid, valid_job_dependencies,
    workflow_shape_valid,
};
use crate::codebase::ci_graph::triggers::CompiledTriggers;
use serde_yaml::Value;
use std::collections::BTreeSet;

mod jobs;
use jobs::{JobScanner, JobStates, WorkflowRuntime};

pub(super) fn scan_activation(
    path: &str,
    document: &WorkflowDocument<'_>,
    triggers: &CompiledTriggers,
    state: &ActivationState,
    context: &ScanContext<'_>,
    memo: &mut ActivationMemo,
) -> Option<BTreeSet<String>> {
    if state.active_paths.contains(path) {
        return None;
    }
    let key = ActivationKey {
        path: path.to_string(),
        state: state.clone(),
    };
    if let Some(result) = memo.get(&key) {
        return result.clone();
    }
    if !memo.try_record_computation() {
        return None;
    }
    let result = scan_activation_uncached(path, document, triggers, state, context, memo);
    memo.insert(key, result.clone());
    result
}

fn scan_activation_uncached(
    path: &str,
    document: &WorkflowDocument<'_>,
    triggers: &CompiledTriggers,
    state: &ActivationState,
    context: &ScanContext<'_>,
    memo: &mut ActivationMemo,
) -> Option<BTreeSet<String>> {
    let mut state = state.clone();
    state.active_paths.insert(path.to_string());
    if !workflow_shape_valid(document.value) {
        return None;
    }
    let jobs = document.value.get("jobs").and_then(Value::as_mapping)?;
    if jobs.is_empty() || !valid_job_dependencies(jobs) {
        return None;
    }
    let job_states = JobStates::new(jobs, &state.inputs)?;
    JobScanner::new(
        &job_states,
        triggers,
        WorkflowRuntime {
            cwd: effective_working_directory(document.value, Some(".".to_string())),
            shell: effective_shell(document.value, None),
            workflow: document.value,
        },
        &state,
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
