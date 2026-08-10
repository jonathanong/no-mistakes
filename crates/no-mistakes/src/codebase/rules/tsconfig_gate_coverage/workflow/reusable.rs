use super::conditions::{
    callee_inputs, callee_secrets_valid, direct_inputs, statically_not_enforcing,
    statically_skipped_jobs, InputState,
};
use super::runtime::{effective_shell, has_static_runnable_runs_on};
use super::{effective_working_directory, ParsedWorkflowSet};
use crate::codebase::ci_graph::{parse::parse_workflow_value, triggers::CompiledTriggers};
use crate::codebase::workflow_topology::workflow_values;
use serde_yaml::Value;
use std::collections::BTreeSet;

use crate::codebase::rules::tsconfig_gate_coverage::ProjectSourceInputs;

mod model;
mod steps;
mod validation;

use model::{ActivationKey, ActivationMemo, ScanContext, WorkflowDocument};
use steps::scan_job_steps;
use validation::{
    call_bindings_shape_valid, reusable_call_job_shape_valid, scan_job_shape_valid,
    valid_job_dependencies, validated_reusable_target, workflow_call_shape_valid,
    zero_instance_matrix,
};

pub(super) fn collect_ci_projects_with_stats(
    parsed: &ParsedWorkflowSet,
    tracked: &BTreeSet<String>,
    project_source_inputs: &ProjectSourceInputs,
) -> (BTreeSet<String>, usize) {
    let workflows = parsed
        .documents
        .iter()
        .filter_map(|document| {
            let value = document.value.as_ref().ok()?;
            Some((
                document.path.clone(),
                WorkflowDocument {
                    value,
                    call_contract: workflow_values::parse_workflow_call(value.get("on")),
                    call_contract_shape_valid: workflow_call_shape_valid(value.get("on")),
                },
            ))
        })
        .collect();
    let context = ScanContext {
        workflows,
        tracked,
        project_source_inputs,
    };
    let mut projects = BTreeSet::new();
    let mut computations = 0;
    for (path, document) in &context.workflows {
        if !document.call_contract_shape_valid {
            continue;
        }
        let trigger_model = parse_workflow_value(document.value, path);
        if trigger_model.triggers.events.is_empty() {
            continue;
        }
        let triggers = CompiledTriggers::new(&trigger_model);
        let mut memo = ActivationMemo::new();
        let Some(inputs) = direct_inputs(document.call_contract.as_ref()) else {
            continue;
        };
        if let Some(activation_projects) = scan_activation(
            path,
            document,
            &triggers,
            &inputs,
            &BTreeSet::new(),
            &context,
            &mut memo,
        ) {
            projects.extend(activation_projects);
        }
        computations += memo.computations();
    }
    (projects, computations)
}

fn scan_activation(
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
    let workflow_cwd = effective_working_directory(document.value, Some(".".to_string()));
    let workflow_shell = effective_shell(document.value, None);
    let jobs = document.value.get("jobs").and_then(Value::as_mapping)?;
    if jobs.is_empty() || !valid_job_dependencies(jobs) {
        return None;
    }
    let skipped_jobs = statically_skipped_jobs(jobs, inputs);
    let mut projects = BTreeSet::new();
    for (job_id, job) in jobs {
        if !scan_job_shape_valid(job) {
            return None;
        }
        let job_id = super::normalized_job_id(job_id)?;
        let call_target = match job.get("uses") {
            Some(Value::String(target))
                if reusable_call_job_shape_valid(job) && call_bindings_shape_valid(job) =>
            {
                Some(target.as_str())
            }
            Some(_) => return None,
            None => None,
        };
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
                let callee_inputs = callee_inputs(Some(contract), job, inputs)?;
                Some(scan_activation(
                    callee_path,
                    callee,
                    triggers,
                    &callee_inputs,
                    &active_paths,
                    context,
                    memo,
                )?)
            } else {
                None
            }
        } else {
            None
        };
        if skipped_jobs.contains(&job_id)
            || statically_not_enforcing(job, inputs)
            || zero_instance_matrix(job)
        {
            continue;
        }
        if call_target.is_some() {
            projects.extend(callee_projects.unwrap_or_default());
            continue;
        }
        if !has_static_runnable_runs_on(job) {
            continue;
        }
        projects.extend(scan_job_steps(
            job,
            triggers,
            inputs,
            workflow_cwd.clone(),
            workflow_shell.clone(),
            context,
        ));
    }
    Some(projects)
}
