mod conditions;
mod expressions;
pub(super) mod local_actions;
mod reusable;
mod runtime;

use super::{application::project_finding, command_scan, RuleFinding};
use crate::codebase::ci_workflows::{ParsedWorkflowSet, WorkflowDocumentErrorKind};
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn normalized_job_id(value: &Value) -> Option<String> {
    crate::codebase::workflow_topology::value_primitives::string_value(Some(value))
        .map(|job_id| job_id.to_lowercase())
}

fn complete_expression(value: &str) -> bool {
    expressions::complete_expression_type(value).is_some()
}

fn complete_literal_expression_value(value: &str) -> Option<Value> {
    expressions::complete_literal_expression_value(value)
}

fn complete_expression_may_be_mapping(value: &str) -> bool {
    expressions::complete_expression_may_produce_mapping(value)
}

pub(super) fn ci_typechecked_projects_with_local_actions(
    root: &std::path::Path,
    workflows: &ParsedWorkflowSet,
    tracked: &BTreeSet<String>,
    tracked_paths: &[PathBuf],
    project_source_inputs: &super::ProjectSourceInputs,
    local_actions: &local_actions::LocalActionCatalog,
) -> BTreeSet<String> {
    ci_typechecked_projects_with_local_actions_and_stats(
        root,
        workflows,
        tracked,
        tracked_paths,
        project_source_inputs,
        local_actions,
    )
    .0
}

pub(super) fn ci_typechecked_projects_with_local_actions_and_stats(
    root: &std::path::Path,
    workflows: &ParsedWorkflowSet,
    tracked: &BTreeSet<String>,
    tracked_paths: &[PathBuf],
    project_source_inputs: &super::ProjectSourceInputs,
    local_actions: &local_actions::LocalActionCatalog,
) -> (BTreeSet<String>, usize) {
    reusable::collect_ci_projects_with_local_actions(
        root,
        workflows,
        tracked,
        tracked_paths,
        project_source_inputs,
        local_actions,
    )
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
