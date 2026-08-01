use super::{application::project_finding, command_scan, RuleFinding};
use crate::codebase::ci_workflows::{ParsedWorkflowSet, WorkflowDocumentErrorKind};
use serde_yaml::Value;
use std::collections::BTreeSet;

pub(super) fn ci_typechecked_projects(workflows: &ParsedWorkflowSet) -> BTreeSet<String> {
    let mut projects = BTreeSet::new();
    for document in &workflows.documents {
        let Ok(workflow) = document.value.as_ref() else {
            continue;
        };
        let workflow_cwd = effective_working_directory(workflow, Some(".".to_string()));
        let workflow_shell = effective_shell(workflow, None);
        let Some(jobs) = workflow.get("jobs").and_then(Value::as_mapping) else {
            continue;
        };
        for job in jobs.values() {
            if statically_not_enforcing(job) {
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
                if !is_supported_posix_shell(effective_shell(step, job_shell.clone()).as_deref()) {
                    continue;
                }
                for project in command_scan::scan_shell_for_typechecked_projects(run, &cwd) {
                    if is_repo_relative_project_path(&project) {
                        projects.insert(project);
                    }
                }
            }
        }
    }
    projects
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

fn default_shell(value: &Value) -> Option<&str> {
    value
        .get("defaults")
        .and_then(|defaults| defaults.get("run"))
        .and_then(|run| run.get("shell"))
        .and_then(Value::as_str)
}

/// Returns the most-specific static shell setting. `None` means GitHub
/// Actions' implicit shell, which preserves the rule's existing behavior.
fn effective_shell(value: &Value, fallback: Option<String>) -> Option<String> {
    match value.get("shell").and_then(Value::as_str) {
        Some(shell) => Some(shell.to_string()),
        None => default_shell(value).map(str::to_string).or(fallback),
    }
}

/// Accept GitHub Actions' implicit shell and static POSIX shell forms only.
/// A custom template must invoke `bash` or `sh`, pass the generated script as
/// `{0}`, and use only flags that preserve normal script execution.
fn is_supported_posix_shell(shell: Option<&str>) -> bool {
    let Some(shell) = shell else {
        return true;
    };
    let mut tokens = shell.split_ascii_whitespace();
    let Some(command) = tokens.next() else {
        return false;
    };
    if !matches!(command, "bash" | "sh") {
        return false;
    }
    let args = tokens.collect::<Vec<_>>();
    args.is_empty() || is_execution_preserving_shell_template(command, &args)
}

fn is_execution_preserving_shell_template(command: &str, arguments: &[&str]) -> bool {
    if arguments.last() != Some(&"{0}") {
        return false;
    }
    let options = &arguments[..arguments.len() - 1];
    let mut index = 0;
    while let Some(option) = options.get(index) {
        match *option {
            "--noprofile" | "--norc" if command == "bash" => index += 1,
            option
                if command == "bash"
                    && is_bash_pipefail_option(option)
                    && options.get(index + 1) == Some(&"pipefail") =>
            {
                index += 2;
            }
            option if is_execution_preserving_short_option(option) => index += 1,
            _ => return false,
        }
    }
    true
}

fn is_bash_pipefail_option(option: &str) -> bool {
    let Some(flags) = option.strip_prefix('-') else {
        return false;
    };
    let Some(prefix) = flags.strip_suffix('o') else {
        return false;
    };
    prefix.chars().all(|flag| matches!(flag, 'e' | 'u' | 'x'))
}

/// `-e`, `-u`, and `-x` only affect error handling or diagnostics. `-o` is
/// handled separately so it can be limited to Bash's execution-safe pipefail.
fn is_execution_preserving_short_option(option: &str) -> bool {
    let Some(flags) = option.strip_prefix('-') else {
        return false;
    };
    !flags.is_empty() && flags.chars().all(|flag| matches!(flag, 'e' | 'u' | 'x'))
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

pub(super) fn is_repo_relative_project_path(project: &str) -> bool {
    command_scan::normalize_repo_relative(project).is_some()
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
