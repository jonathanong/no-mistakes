use super::super::conditions::{
    callee_inputs, callee_secrets_valid, inputs_with_matrix_values, statically_not_enforcing,
    statically_skipped_jobs, InputState,
};
use super::super::effective_working_directory;
use super::super::runtime::{
    container_runner_support, effective_shell, has_static_runnable_runs_on, ContainerRunnerSupport,
};
use super::model::{ActivationKey, ActivationMemo, ScanContext, WorkflowDocument};
use super::steps::scan_job_steps;
use super::validation::{
    call_bindings_shape_valid, reusable_call_job_shape_valid, scan_job_shape_valid,
    uniform_static_matrix_values, valid_job_dependencies, validated_reusable_target,
    workflow_shape_valid, zero_instance_matrix,
};
use crate::codebase::ci_graph::triggers::CompiledTriggers;
use crate::codebase::workflow_topology::workflow_values;
use serde_yaml::Value;
use std::collections::BTreeSet;

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
    let zero_instance_jobs = zero_instance_job_ids(jobs)?;
    let skipped_jobs = statically_skipped_jobs(jobs, inputs, &zero_instance_jobs);
    let mut projects = BTreeSet::new();
    for (job_id, job) in jobs {
        if !scan_job_shape_valid(job) {
            return None;
        }
        let job_id = super::super::normalized_job_id(job_id)?;
        let call_target = reusable_call_target(job)?;
        let job_skipped = skipped_jobs.contains(&job_id)
            || statically_not_enforcing(job, inputs)
            || zero_instance_matrix(job);
        let callee_projects = if let Some(target) = call_target {
            let edge = workflow_values::call_edge(&job_id, target, job);
            if !memo.register_target(validated_reusable_target(&edge)?) {
                return None;
            }
            if edge.local {
                let callee_path = edge.to.as_deref().unwrap_or_default();
                let callee = context.workflows.get(callee_path)?;
                if !callee.call_contract_shape_valid {
                    return None;
                }
                let contract = callee.call_contract.as_ref()?;
                if active_paths.len() == 10 || !callee_secrets_valid(contract, job) {
                    return None;
                }
                let matrix_values = uniform_static_matrix_values(job);
                let matrix_inputs = inputs_with_matrix_values(inputs, &matrix_values);
                let callee_inputs = callee_inputs(Some(contract), job, &matrix_inputs)?;
                if job_skipped {
                    Some(BTreeSet::new())
                } else {
                    Some(scan_activation(
                        callee_path,
                        callee,
                        triggers,
                        &callee_inputs,
                        &active_paths,
                        context,
                        memo,
                    )?)
                }
            } else {
                None
            }
        } else {
            None
        };
        if job_skipped {
            continue;
        }
        if call_target.is_some() {
            projects.extend(callee_projects.unwrap_or_default());
            continue;
        }
        if step_job_runner_supported(job) {
            projects.extend(scan_job_steps(
                job,
                triggers,
                inputs,
                workflow_cwd.clone(),
                workflow_shell.clone(),
                context,
            ));
        }
    }
    Some(projects)
}

fn zero_instance_job_ids(jobs: &serde_yaml::Mapping) -> Option<BTreeSet<String>> {
    jobs.iter()
        .filter(|(_, job)| zero_instance_matrix(job))
        .map(|(job_id, _)| super::super::normalized_job_id(job_id))
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
