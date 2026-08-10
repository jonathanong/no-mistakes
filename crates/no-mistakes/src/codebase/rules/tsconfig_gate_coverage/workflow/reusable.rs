use super::conditions::{
    callee_inputs, direct_inputs, statically_not_enforcing, statically_skipped_jobs, InputState,
};
use super::runtime::{effective_shell, has_static_runnable_runs_on};
use super::{effective_working_directory, ParsedWorkflowSet};
use crate::codebase::ci_graph::{parse::parse_workflow_value, triggers::CompiledTriggers};
use crate::codebase::workflow_topology::{model::WorkflowCallContract, workflow_values};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

use crate::codebase::rules::tsconfig_gate_coverage::ProjectSourceInputs;

mod steps;

use steps::scan_job_steps;

struct WorkflowDocument<'a> {
    value: &'a Value,
    call_contract: Option<WorkflowCallContract>,
}

struct ScanContext<'a> {
    workflows: BTreeMap<String, WorkflowDocument<'a>>,
    tracked: &'a BTreeSet<String>,
    project_source_inputs: &'a ProjectSourceInputs,
}

pub(super) fn collect_ci_projects(
    parsed: &ParsedWorkflowSet,
    tracked: &BTreeSet<String>,
    project_source_inputs: &ProjectSourceInputs,
) -> BTreeSet<String> {
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
    for (path, document) in &context.workflows {
        let trigger_model = parse_workflow_value(document.value, path);
        if trigger_model.triggers.events.is_empty() {
            continue;
        }
        let triggers = CompiledTriggers::new(&trigger_model);
        let inputs = direct_inputs(document.call_contract.as_ref());
        projects.extend(scan_activation(
            path,
            document,
            &triggers,
            &inputs,
            &BTreeSet::new(),
            &context,
        ));
    }
    projects
}

fn scan_activation(
    path: &str,
    document: &WorkflowDocument<'_>,
    triggers: &CompiledTriggers,
    inputs: &InputState,
    active_paths: &BTreeSet<String>,
    context: &ScanContext<'_>,
) -> BTreeSet<String> {
    if active_paths.contains(path) {
        return BTreeSet::new();
    }
    let mut active_paths = active_paths.clone();
    active_paths.insert(path.to_string());
    let workflow_cwd = effective_working_directory(document.value, Some(".".to_string()));
    let workflow_shell = effective_shell(document.value, None);
    let Some(jobs) = document.value.get("jobs").and_then(Value::as_mapping) else {
        return BTreeSet::new();
    };
    let skipped_jobs = statically_skipped_jobs(jobs, inputs);
    let mut projects = BTreeSet::new();
    for (job_id, job) in jobs {
        if job_id
            .as_str()
            .is_some_and(|job_id| skipped_jobs.contains(job_id))
            || statically_not_enforcing(job, inputs)
        {
            continue;
        }
        if let Some(target) = job.get("uses").and_then(Value::as_str) {
            let edge = workflow_values::call_edge(job_id.as_str().unwrap_or(""), target, job);
            if !edge.local {
                continue;
            }
            let callee_path = edge.to.as_deref().unwrap_or_default();
            let Some(callee) = context.workflows.get(callee_path) else {
                continue;
            };
            let Some(contract) = callee.call_contract.as_ref() else {
                continue;
            };
            let Some(callee_inputs) = callee_inputs(Some(contract), job, inputs) else {
                continue;
            };
            projects.extend(scan_activation(
                callee_path,
                callee,
                triggers,
                &callee_inputs,
                &active_paths,
                context,
            ));
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
    projects
}
