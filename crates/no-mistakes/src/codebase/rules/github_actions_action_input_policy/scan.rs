use super::lines::input_line;
use super::policy::{self, Compiled, InputFault};
use super::{RuleFinding, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use serde_yaml::{Mapping, Value};
use std::path::Path;

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    opts: &Compiled,
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    let Some(source) = super::super::read_language_source(sources, path) else {
        return Vec::new();
    };
    if !defer_suppression && crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
    {
        return Vec::new();
    }
    let mut findings = match serde_yaml::from_str::<Value>(&source) {
        Ok(value) => check_parsed(&rel, &source, &value, opts),
        Err(_) => Vec::new(),
    };
    if !defer_suppression {
        super::super::suppress_rule_findings_with_source(&mut findings, &source);
    }
    findings
}

pub(super) fn check_parsed(
    rel: &str,
    source: &str,
    value: &Value,
    opts: &Compiled,
) -> Vec<RuleFinding> {
    let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    for (key, job) in jobs {
        let Some(job_id) = key.as_str() else {
            continue;
        };
        let Some(job) = job.as_mapping() else {
            continue;
        };
        let Some(steps) = mapping_get(job, "steps").and_then(Value::as_sequence) else {
            continue;
        };
        for (index, step) in steps.iter().enumerate() {
            let Some(step) = step.as_mapping() else {
                continue;
            };
            findings.extend(check_step(rel, source, job_id, step, index, opts));
        }
    }
    findings
}

fn check_step(
    rel: &str,
    source: &str,
    job_id: &str,
    step: &Mapping,
    index: usize,
    opts: &Compiled,
) -> Vec<RuleFinding> {
    let Some(uses) = mapping_string(step, "uses") else {
        return Vec::new();
    };
    let Some(family) = policy::matching(&uses, &opts.actions) else {
        return Vec::new();
    };
    let label = step_label(step, &uses);
    let with = mapping_get(step, "with").and_then(Value::as_mapping);
    policy::judge(with, family)
        .into_iter()
        .map(|fault| {
            let key = fault_key(&fault);
            finding(
                rel,
                input_line(source, job_id, index, key),
                message(rel, job_id, &label, &family.name, &fault),
                job_id,
            )
        })
        .collect()
}

fn fault_key(fault: &InputFault) -> &str {
    match fault {
        InputFault::Missing { key, .. }
        | InputFault::Mismatch { key, .. }
        | InputFault::Banned { key } => key,
    }
}

fn message(rel: &str, job_id: &str, label: &str, family: &str, fault: &InputFault) -> String {
    let where_ = format!("{rel}: job \"{job_id}\" step \"{label}\" calls `{family}`");
    match fault {
        InputFault::Missing { key, expected } => format!(
            "{where_} without required input `{key}`. Set `{key}` to {expected}. The action policy requires this value on every ref."
        ),
        InputFault::Mismatch { key, expected, got } => format!(
            "{where_} with `{key}` set to {got}. Set `{key}` to {expected}. The action policy requires this value on every ref."
        ),
        InputFault::Banned { key } => format!(
            "{where_} and sets banned input `{key}`. Remove `{key}` from `with:`."
        ),
    }
}

fn finding(file: &str, line: usize, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}

fn step_label(step: &Mapping, uses: &str) -> String {
    mapping_string(step, "name")
        .or_else(|| mapping_string(step, "id"))
        .unwrap_or_else(|| uses.to_string())
}

fn mapping_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_string()))
}

fn mapping_string(step: &Mapping, key: &str) -> Option<String> {
    mapping_get(step, key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
