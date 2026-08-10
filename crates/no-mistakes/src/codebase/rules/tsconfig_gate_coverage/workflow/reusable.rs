use super::conditions::{
    callee_inputs, direct_inputs, statically_not_enforcing, statically_skipped_jobs, InputState,
};
use super::runtime::{
    effective_shell, has_static_runnable_runs_on, runs_on_can_default_to_windows,
    shell_failure_enforced,
};
use super::{effective_working_directory, ParsedWorkflowSet};
use crate::codebase::ci_graph::{
    parse::parse_workflow_value,
    triggers::{CompiledTriggers, TriggerMatch},
};
use crate::codebase::workflow_topology::{model::WorkflowCallContract, workflow_values};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

use crate::codebase::rules::tsconfig_gate_coverage::{
    application::resolve_gate_project_against_tracked, command_scan, ProjectSourceInputs,
};

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
        if let Some(activation_projects) = scan_activation(
            path,
            document,
            &triggers,
            &inputs,
            &BTreeSet::new(),
            &context,
        ) {
            projects.extend(activation_projects);
        }
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
) -> Option<BTreeSet<String>> {
    if active_paths.contains(path) {
        return None;
    }
    let mut active_paths = active_paths.clone();
    active_paths.insert(path.to_string());
    let workflow_cwd = effective_working_directory(document.value, Some(".".to_string()));
    let workflow_shell = effective_shell(document.value, None);
    let Some(jobs) = document.value.get("jobs").and_then(Value::as_mapping) else {
        return Some(BTreeSet::new());
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
            let Some(callee_path) = edge.to.as_deref() else {
                continue;
            };
            let Some(callee) = context.workflows.get(callee_path) else {
                continue;
            };
            let Some(contract) = callee.call_contract.as_ref() else {
                continue;
            };
            let Some(callee_inputs) = callee_inputs(Some(contract), job, inputs) else {
                continue;
            };
            if let Some(callee_projects) = scan_activation(
                callee_path,
                callee,
                triggers,
                &callee_inputs,
                &active_paths,
                context,
            ) {
                projects.extend(callee_projects);
            }
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

fn scan_job_steps(
    job: &Value,
    triggers: &CompiledTriggers,
    inputs: &InputState,
    workflow_cwd: Option<String>,
    workflow_shell: Option<String>,
    context: &ScanContext<'_>,
) -> BTreeSet<String> {
    let Some(steps) = job.get("steps").and_then(Value::as_sequence) else {
        return BTreeSet::new();
    };
    let job_cwd = effective_working_directory(job, workflow_cwd);
    let job_shell = effective_shell(job, workflow_shell);
    let mut projects = BTreeSet::new();
    for step in steps {
        if statically_not_enforcing(step, inputs) {
            continue;
        }
        let step_cwd = match step.get("working-directory").and_then(Value::as_str) {
            Some(raw) => command_scan::normalize_repo_relative(raw),
            None => job_cwd.clone(),
        };
        let Some(cwd) = step_cwd else {
            continue;
        };
        let Some(run) = step.get("run").and_then(Value::as_str) else {
            continue;
        };
        let shell = effective_shell(step, job_shell.clone());
        if shell.is_none() && runs_on_can_default_to_windows(job) {
            continue;
        }
        let Some(failure_enforced) = shell_failure_enforced(shell.as_deref()) else {
            continue;
        };
        let scanned = if failure_enforced {
            command_scan::scan_shell_for_typechecked_projects(run, &cwd)
        } else {
            command_scan::scan_workflow_shell_for_typechecked_projects(run, &cwd, false)
        };
        for project in scanned {
            let project = resolve_gate_project_against_tracked(&project, context.tracked);
            if context
                .project_source_inputs
                .get(&project)
                .is_some_and(|source_inputs| {
                    source_inputs.iter().all(|input| {
                        matches!(
                            triggers.evaluate(input).0,
                            TriggerMatch::Matched | TriggerMatch::Always
                        )
                    })
                })
            {
                projects.insert(project);
            }
        }
    }
    projects
}
