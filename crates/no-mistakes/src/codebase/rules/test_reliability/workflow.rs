use super::{Location, Origin, Shape};
use crate::codebase::rules::github_actions_workflow_files::step_lines::{walk_steps, Job};
use crate::codebase::ts_source::SourceStore;
use serde_yaml::Value;
use std::path::Path;

mod shell;

pub(super) fn collect(
    path: &Path,
    sources: &SourceStore,
    commands: &[Vec<String>],
) -> anyhow::Result<Vec<Location>> {
    let document = sources.parse_yaml_path(path)?;
    let source = sources.read_path(path)?;
    let Some(jobs) = document.get("jobs").and_then(Value::as_mapping) else {
        return Ok(Vec::new());
    };
    let locations = walk_steps(&source, false);
    let mut findings = Vec::new();
    for (name, job) in jobs {
        let Some(steps) = job.get("steps").and_then(Value::as_sequence) else {
            continue;
        };
        let job_locations = name
            .as_str()
            .and_then(|name| locations.iter().find(|job| job.name == name));
        for (index, step) in steps.iter().enumerate() {
            if let Some(retries) = retry_action(step) {
                findings.push(Location {
                    line: step_line(job_locations, index, "uses"),
                    shape: Shape::Retry("GitHub Actions retry action", retries),
                    project: None,
                    origin: Origin::Settings,
                    playwright_import: None,
            playwright_candidates: Vec::new(),
                    playwright_binding: None,
                    playwright_wrappers: Vec::new(),
                    resolved_playwright_exports: Vec::new(),
                    unresolved_reason: None,
                });
            }
            let Some(script) = step.get("run").and_then(Value::as_str) else {
                continue;
            };
            if !commands.is_empty() && is_supported_shell(&document, job, step) {
                for retries in shell::rerun_loops(script, commands) {
                    findings.push(Location {
                        line: step_line(job_locations, index, "run"),
                        shape: Shape::Retry("GitHub Actions test rerun loop", retries),
                        project: None,
                        origin: Origin::Settings,
                        playwright_import: None,
            playwright_candidates: Vec::new(),
                        playwright_binding: None,
                        playwright_wrappers: Vec::new(),
                        resolved_playwright_exports: Vec::new(),
                        unresolved_reason: None,
                    });
                }
            }
        }
    }
    Ok(findings)
}

fn retry_action(step: &Value) -> Option<Option<u64>> {
    let (action, _) = step.get("uses")?.as_str()?.split_once('@')?;
    let (input, default) = match action.to_ascii_lowercase().as_str() {
        "nick-fields/retry" => ("max_attempts", 3),
        "wandalen/wretry.action"
        | "wandalen/wretry.action/pre"
        | "wandalen/wretry.action/main"
        | "wandalen/wretry.action/post" => ("attempt_limit", 2),
        _ => return None,
    };
    let attempts = match step.get("with").and_then(|with| with.get(input)) {
        Some(value) => value
            .as_u64()
            .or_else(|| value.as_str()?.trim().parse::<u64>().ok()),
        None => Some(default),
    };
    Some(attempts.and_then(|attempts| attempts.checked_sub(1)))
}

fn is_supported_shell(workflow: &Value, job: &Value, step: &Value) -> bool {
    let shell = step
        .get("shell")
        .and_then(Value::as_str)
        .or_else(|| default_shell(job))
        .or_else(|| default_shell(workflow));
    match shell.and_then(|shell| shell.split_whitespace().next()) {
        Some(program) => matches!(program, "bash" | "sh"),
        None => !job.get("runs-on").is_some_and(windows_runner),
    }
}

fn default_shell(value: &Value) -> Option<&str> {
    value.get("defaults")?.get("run")?.get("shell")?.as_str()
}

fn windows_runner(runners: &Value) -> bool {
    fn label(value: &Value) -> bool {
        value.as_str().is_some_and(|label| {
            let label = label.to_ascii_lowercase();
            label == "windows" || label.starts_with("windows-")
        })
    }
    let labels = runners.get("labels").unwrap_or(runners);
    label(labels)
        || labels
            .as_sequence()
            .is_some_and(|labels| labels.iter().any(label))
}

fn step_line(job: Option<&Job>, index: usize, key: &str) -> usize {
    let Some(step) = job.and_then(|job| job.steps.get(index)) else {
        return 1;
    };
    step.fields
        .iter()
        .find(|field| field.name == key)
        .map_or(step.line, |field| field.line)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
