use super::super::effective_working_directory;
use super::super::runtime::{
    container_runner_support, effective_shell, has_static_runnable_runs_on, ContainerRunnerSupport,
};
use super::model::{
    ActivationKey, ActivationMemo, ActivationScan, ActivationState, ScanContext, WorkflowDocument,
};
use super::validation::{
    call_bindings_shape_valid, reusable_call_job_shape_valid, valid_job_dependencies,
    workflow_concurrency_valid_for_inputs, workflow_shape_valid,
};
use crate::codebase::ci_graph::triggers::CompiledTriggers;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    complete_expression_static_value, StaticValue,
};
use serde_yaml::Value;
use std::collections::BTreeMap;

mod job_states;
mod jobs;
use job_states::JobStates;
use jobs::{JobScanner, WorkflowRuntime};

pub(super) fn scan_activation(
    path: &str,
    document: &WorkflowDocument<'_>,
    triggers: &CompiledTriggers,
    state: &ActivationState,
    context: &ScanContext<'_>,
    memo: &mut ActivationMemo,
) -> Option<ActivationScan> {
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
) -> Option<ActivationScan> {
    let mut state = state.clone();
    state.active_paths.insert(path.to_string());
    if !workflow_shape_valid(document.value) {
        return None;
    }
    if !workflow_concurrency_valid_for_inputs(document.value.get("concurrency"), &state.inputs) {
        return None;
    }
    let jobs = document.value.get("jobs").and_then(Value::as_mapping)?;
    if jobs.is_empty() || !valid_job_dependencies(jobs) {
        return None;
    }
    let job_states = JobStates::new(jobs, &state.inputs)?;
    let mut scan = JobScanner::new(
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
    .scan(jobs)?;
    scan.outputs = static_workflow_outputs(document, &state.inputs);
    Some(scan)
}

fn static_workflow_outputs(
    document: &WorkflowDocument<'_>,
    inputs: &super::super::conditions::InputState,
) -> BTreeMap<String, StaticValue> {
    document
        .call_contract
        .as_ref()
        .into_iter()
        .flat_map(|contract| contract.outputs.iter())
        .filter_map(|(name, output)| {
            let value = complete_expression_static_value(output.value.as_deref()?, inputs)?;
            value.function_string().map(|value| {
                // Workflow outputs cross the reusable boundary as strings;
                // preserve only projections every caller can compare exactly.
                (name.to_lowercase(), StaticValue::String(value))
            })
        })
        .collect()
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

fn step_job_runner_supported(job: &Value, inputs: &super::super::conditions::InputState) -> bool {
    if !has_static_runnable_runs_on(job, inputs) {
        return false;
    }
    let requires_linux_runner = job.get("container").is_some() || job.get("services").is_some();
    !requires_linux_runner
        || job.as_mapping().is_some_and(|job| {
            container_runner_support(job, inputs) == ContainerRunnerSupport::Linux
        })
}
