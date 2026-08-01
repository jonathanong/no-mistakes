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
                let Some(failure_enforced) =
                    shell_failure_enforced(effective_shell(step, job_shell.clone()).as_deref())
                else {
                    continue;
                };
                let scanned_projects = if failure_enforced {
                    command_scan::scan_shell_for_typechecked_projects(run, &cwd)
                } else {
                    command_scan::scan_workflow_shell_for_typechecked_projects(run, &cwd, false)
                };
                for project in scanned_projects {
                    projects.insert(project);
                }
            }
        }
    }
    projects
}

/// A CI job cannot provide a typecheck gate unless Actions can schedule it on
/// a statically known runner. Reusable-workflow jobs use `uses:` rather than
/// `steps:` and are already excluded by the step requirement above.
fn has_static_runnable_runs_on(job: &Value) -> bool {
    match job.get("runs-on") {
        Some(Value::String(label)) => is_static_runner_label(label),
        Some(Value::Sequence(labels)) => {
            !labels.is_empty()
                && labels
                    .iter()
                    .all(|label| label.as_str().is_some_and(is_static_runner_label))
        }
        _ => false,
    }
}

fn is_static_runner_label(label: &str) -> bool {
    !label.trim().is_empty() && !label.contains("${{")
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

/// Return whether a supported shell preserves failures for every command in a
/// multi-command body. Built-in and implicit Actions shells provide `-e`; a
/// custom template must express `-e` or `-o errexit` itself.
fn shell_failure_enforced(shell: Option<&str>) -> Option<bool> {
    let Some(shell) = shell else {
        return Some(true);
    };
    let mut tokens = shell.split_ascii_whitespace();
    let command = tokens.next()?;
    if !matches!(command, "bash" | "sh") {
        return None;
    }
    let args = tokens.collect::<Vec<_>>();
    args.is_empty()
        .then_some(true)
        .or_else(|| execution_preserving_shell_template_failure_enforced(command, &args))
}

fn execution_preserving_shell_template_failure_enforced(
    command: &str,
    arguments: &[&str],
) -> Option<bool> {
    if arguments.last() != Some(&"{0}") {
        return None;
    }
    let options = &arguments[..arguments.len() - 1];
    let mut index = 0;
    let mut failure_enforced = false;
    while let Some(option) = options.get(index) {
        match *option {
            "--noprofile" | "--norc" if command == "bash" => index += 1,
            option
                if command == "bash"
                    && is_bash_pipefail_option(option)
                    && options.get(index + 1) == Some(&"pipefail") =>
            {
                failure_enforced |= option.contains('e');
                index += 2;
            }
            "-o" if options.get(index + 1) == Some(&"errexit") => {
                failure_enforced = true;
                index += 2;
            }
            option if let Some(enforced) = execution_preserving_short_option(option) => {
                failure_enforced |= enforced;
                index += 1;
            }
            _ => return None,
        }
    }
    Some(failure_enforced)
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
fn execution_preserving_short_option(option: &str) -> Option<bool> {
    let flags = option.strip_prefix('-')?;
    (!flags.is_empty() && flags.chars().all(|flag| matches!(flag, 'e' | 'u' | 'x')))
        .then_some(flags.contains('e'))
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
