use super::options::CompiledOptions;
use super::scan_ci_shell::find_shell_reruns;
use super::RULE_ID;
use crate::codebase::rules::github_actions_script_expressions::step_run_lines;
use crate::codebase::rules::github_actions_workflow_files::{
    run_scripts::run_scripts, script_line::yaml_line, step_lines::first_key_line,
};
use crate::codebase::rules::{self, RuleFinding};
use crate::codebase::ts_source::SourceStore;
use serde_yaml::{Mapping, Value};
use std::path::Path;

pub(super) fn scan_file(
    path: &Path,
    rel: &str,
    action: bool,
    opts: &CompiledOptions,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    if opts.allow.is_allowed(rel, None) {
        return Vec::new();
    }
    let Some(source) = rules::read_language_source(sources, path) else {
        return Vec::new();
    };
    scan_source(rel, &source, action, opts)
}

pub(super) fn scan_source(
    rel: &str,
    source: &str,
    action: bool,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    let source = source.trim_start_matches('\u{feff}');
    let Ok(value) = serde_yaml::from_str::<Value>(source) else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    scan_retry_actions(rel, source, action, &value, opts, &mut findings);
    if !opts.ci_test_commands.is_empty() {
        scan_rerun_scripts(rel, source, action, &value, opts, &mut findings);
    }
    findings
}

fn scan_retry_actions(
    rel: &str,
    source: &str,
    action: bool,
    value: &Value,
    opts: &CompiledOptions,
    findings: &mut Vec<RuleFinding>,
) {
    if action {
        if let Some(steps) = value
            .get("runs")
            .and_then(|runs| runs.get("steps"))
            .and_then(Value::as_sequence)
        {
            check_steps(rel, source, "(composite)", steps, opts, findings);
        }
        return;
    }
    let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) else {
        return;
    };
    for (job_key, job_val) in jobs {
        let Some(job_id) = job_key.as_str() else {
            continue;
        };
        let Some(steps) = job_val
            .as_mapping()
            .and_then(|map| map.get(Value::String("steps".to_string())))
            .and_then(Value::as_sequence)
        else {
            continue;
        };
        check_steps(rel, source, job_id, steps, opts, findings);
    }
}

fn check_steps(
    rel: &str,
    source: &str,
    job_id: &str,
    steps: &[Value],
    opts: &CompiledOptions,
    findings: &mut Vec<RuleFinding>,
) {
    for (index, step) in steps.iter().enumerate() {
        let Some(map) = step.as_mapping() else {
            continue;
        };
        let Some(uses) = map
            .get(Value::String("uses".to_string()))
            .and_then(Value::as_str)
        else {
            continue;
        };
        if !opts.matches_retry_action(uses) {
            continue;
        }
        let with = map
            .get(Value::String("with".to_string()))
            .and_then(Value::as_mapping);
        if let Some(attempts) = with.and_then(configured_attempts) {
            if attempts.saturating_sub(1) <= opts.max_retries {
                continue;
            }
        }
        let line = first_key_line(source, job_id, index, &["uses"]);
        let action_name = uses.trim().split('@').next().unwrap_or(uses.trim());
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: rel.to_string(),
            line,
            message: format!(
                "CI step uses retry action `{action_name}` (maxRetries is {}). Retries hide flaky tests; remove the retry wrapper and fix the flake, or add an allow entry with a reason.",
                opts.max_retries
            ),
            import: None,
            target: Some(action_name.to_string()),
            causes: Vec::new(),
        });
    }
}

fn configured_attempts(with: &Mapping) -> Option<u64> {
    for key in ["max_attempts", "attempt_limit"] {
        let Some(val) = with.get(Value::String(key.to_string())) else {
            continue;
        };
        if let Some(num) = val.as_u64() {
            return Some(num);
        }
        if let Some(text) = val.as_str().and_then(|s| s.trim().parse::<u64>().ok()) {
            return Some(text);
        }
    }
    None
}

fn scan_rerun_scripts(
    rel: &str,
    source: &str,
    action: bool,
    value: &Value,
    opts: &CompiledOptions,
    findings: &mut Vec<RuleFinding>,
) {
    let located = step_run_lines(source, action);
    for body in run_scripts(action, value) {
        let step_lines = located
            .iter()
            .find(|item| item.job == body.job && item.step == body.step);
        for hit in find_shell_reruns(&body.script, &opts.ci_test_commands) {
            let line = yaml_line(step_lines, &body.script, hit.line, &hit.token);
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.to_string(),
                line,
                message: format!(
                    "CI `run` script reruns configured test command `{}` via {}. Remove the shell rerun and fix the flake, or add an allow entry with a reason.",
                    hit.command, hit.shape
                ),
                import: None,
                target: Some(hit.command),
                causes: Vec::new(),
            });
        }
    }
}
