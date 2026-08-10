mod conditions;
mod reusable;
mod runtime;

use super::{application::project_finding, command_scan, RuleFinding};
use crate::codebase::ci_workflows::{ParsedWorkflowSet, WorkflowDocumentErrorKind};
use serde_yaml::Value;
use std::collections::BTreeSet;

pub(super) fn ci_typechecked_projects(
    workflows: &ParsedWorkflowSet,
    tracked: &BTreeSet<String>,
    project_source_inputs: &super::ProjectSourceInputs,
) -> BTreeSet<String> {
    ci_typechecked_projects_with_stats(workflows, tracked, project_source_inputs).0
}

fn normalized_job_id(value: &Value) -> Option<String> {
    crate::codebase::workflow_topology::value_primitives::string_value(Some(value))
        .map(|job_id| job_id.to_lowercase())
}

fn complete_expression(value: &str) -> bool {
    let value = value.trim();
    let Some(body) = value
        .strip_prefix("${{")
        .and_then(|body| body.strip_suffix("}}"))
    else {
        return false;
    };
    let body = body.trim();
    !body.is_empty() && expression_body_delimiters_valid(body)
}

fn expression_body_delimiters_valid(body: &str) -> bool {
    let mut characters = body.chars().peekable();
    let mut in_string = false;
    while let Some(character) = characters.next() {
        if character == '\'' {
            if in_string && characters.peek() == Some(&'\'') {
                characters.next();
            } else {
                in_string = !in_string;
            }
        } else if !in_string && matches!(character, '{' | '}') {
            return false;
        }
    }
    !in_string
}

pub(super) fn ci_typechecked_projects_with_stats(
    workflows: &ParsedWorkflowSet,
    tracked: &BTreeSet<String>,
    project_source_inputs: &super::ProjectSourceInputs,
) -> (BTreeSet<String>, usize) {
    reusable::collect_ci_projects_with_stats(workflows, tracked, project_source_inputs)
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
