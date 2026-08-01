mod runtime;

use super::{
    application::{project_finding, resolve_gate_project_against_tracked},
    command_scan, RuleFinding,
};
use crate::codebase::ci_graph::{
    parse::parse_workflow_value,
    triggers::{CompiledTriggers, TriggerMatch},
};
use crate::codebase::ci_workflows::{ParsedWorkflowSet, WorkflowDocumentErrorKind};
use runtime::{
    effective_shell, has_static_runnable_runs_on, runs_on_can_default_to_windows,
    shell_failure_enforced,
};
use serde_yaml::Value;
use std::collections::BTreeSet;

pub(super) fn ci_typechecked_projects(
    workflows: &ParsedWorkflowSet,
    tracked: &BTreeSet<String>,
    project_source_inputs: &super::ProjectSourceInputs,
) -> BTreeSet<String> {
    let mut projects = BTreeSet::new();
    for document in &workflows.documents {
        let Ok(workflow) = document.value.as_ref() else {
            continue;
        };
        let trigger_model = parse_workflow_value(workflow, &document.path);
        let triggers = CompiledTriggers::new(&trigger_model);
        let workflow_cwd = effective_working_directory(workflow, Some(".".to_string()));
        let workflow_shell = effective_shell(workflow, None);
        let Some(jobs) = workflow.get("jobs").and_then(Value::as_mapping) else {
            continue;
        };
        let skipped_jobs = statically_skipped_jobs(jobs);
        for (job_id, job) in jobs {
            if job_id
                .as_str()
                .is_some_and(|job_id| skipped_jobs.contains(job_id))
            {
                continue;
            }
            if statically_not_enforcing(job) || !has_static_runnable_runs_on(job) {
                continue;
            }
            let Some(steps) = job.get("steps").and_then(Value::as_sequence) else {
                continue;
            };
            let job_cwd = effective_working_directory(job, workflow_cwd.clone());
            let job_shell = effective_shell(job, workflow_shell.clone());
            for step in steps {
                if statically_not_enforcing(step) {
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
                let scanned_projects = if failure_enforced {
                    command_scan::scan_shell_for_typechecked_projects(run, &cwd)
                } else {
                    command_scan::scan_workflow_shell_for_typechecked_projects(run, &cwd, false)
                };
                for project in scanned_projects {
                    let project = resolve_gate_project_against_tracked(&project, tracked);
                    if project_source_inputs.get(&project).is_some_and(|inputs| {
                        inputs.iter().all(|input| {
                            matches!(
                                triggers.evaluate(input).0,
                                TriggerMatch::Matched | TriggerMatch::Always
                            )
                        })
                    }) {
                        projects.insert(project);
                    }
                }
            }
        }
    }
    projects
}

fn statically_skipped_jobs(jobs: &serde_yaml::Mapping) -> BTreeSet<String> {
    let mut skipped = BTreeSet::new();
    loop {
        let mut changed = false;
        for (job_id, job) in jobs {
            let Some(job_id) = job_id.as_str() else {
                continue;
            };
            let directly_disabled = static_bool(job.get("if")) == Some(false);
            let blocked_by_need = !continues_after_skipped_need(job)
                && crate::codebase::workflow_topology::value_primitives::string_list(
                    job.get("needs"),
                )
                .iter()
                .any(|need| skipped.contains(need));
            if (directly_disabled || blocked_by_need) && skipped.insert(job_id.to_string()) {
                changed = true;
            }
        }
        if !changed {
            return skipped;
        }
    }
}

fn continues_after_skipped_need(job: &Value) -> bool {
    job.get("if")
        .and_then(Value::as_str)
        .is_some_and(|expression| {
            matches!(
                expression.trim(),
                "always()" | "${{ always() }}" | "!cancelled()" | "${{ !cancelled() }}"
            )
        })
}

/// A static disabled or non-blocking YAML node cannot enforce a typecheck.
/// Only exact boolean expressions are resolved; all other expressions remain
/// unresolved so the rule stays deterministic.
fn statically_not_enforcing(value: &Value) -> bool {
    static_bool(value.get("if")) == Some(false)
        || static_bool(value.get("continue-on-error")) == Some(true)
}

fn static_bool(value: Option<&Value>) -> Option<bool> {
    match value {
        Some(Value::Bool(value)) => Some(*value),
        Some(Value::String(expression)) => match expression.trim() {
            "${{ false }}" => Some(false),
            "${{ true }}" => Some(true),
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn default_working_directory(value: &Value) -> Option<&str> {
    value
        .get("defaults")
        .and_then(|defaults| defaults.get("run"))
        .and_then(|run| run.get("working-directory"))
        .and_then(Value::as_str)
}

pub(super) fn effective_working_directory(
    value: &Value,
    fallback: Option<String>,
) -> Option<String> {
    match default_working_directory(value) {
        Some(raw) => command_scan::normalize_repo_relative(raw),
        None => fallback,
    }
}

pub(crate) fn workflow_load_findings(workflows: &ParsedWorkflowSet) -> Vec<RuleFinding> {
    let mut findings = workflows
        .documents
        .iter()
        .filter_map(|document| document.value.as_ref().err().map(|error| (document, error)))
        .map(|(document, error)| {
            let detail = match error.kind {
                WorkflowDocumentErrorKind::Read => "could not read workflow file",
                WorkflowDocumentErrorKind::Parse => "could not parse workflow YAML",
            };
            project_finding(
                &document.path,
                format!("{}: {detail}: {}", document.path, error.message),
            )
        })
        .collect::<Vec<_>>();
    findings.sort();
    findings
}
