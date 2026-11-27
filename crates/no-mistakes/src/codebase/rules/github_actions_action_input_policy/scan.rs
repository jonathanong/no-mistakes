use super::lines::{input_line, job_input_line};
use super::message::{self, Site};
use super::policy::{self, Compiled};
use super::RULE_ID;
use no_mistakes_rule_kernel::finding::RuleFinding;
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
        super::super::suppress_rule_findings_for_path(sources, path, &mut findings);
    }
    findings
}

pub(super) fn check_parsed(
    rel: &str,
    source: &str,
    value: &Value,
    opts: &Compiled,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    if let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) {
        findings.extend(check_jobs(rel, source, jobs, opts));
    }
    findings.extend(check_composite(rel, source, value, opts));
    findings
}

fn check_jobs(rel: &str, source: &str, jobs: &Mapping, opts: &Compiled) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for (key, job) in jobs {
        let Some(job_id) = key.as_str() else {
            continue;
        };
        let Some(job) = job.as_mapping() else {
            continue;
        };
        if let Some(steps) = mapping_get(job, "steps").and_then(Value::as_sequence) {
            for (index, step) in steps.iter().enumerate() {
                let Some(step) = step.as_mapping() else {
                    continue;
                };
                findings.extend(judge_call(
                    rel,
                    source,
                    step,
                    Site::Step { job_id, index },
                    opts,
                ));
            }
        }
        findings.extend(judge_call(rel, source, job, Site::Job { job_id }, opts));
    }
    findings
}

fn check_composite(rel: &str, source: &str, value: &Value, opts: &Compiled) -> Vec<RuleFinding> {
    let Some(runs) = value.get("runs").and_then(Value::as_mapping) else {
        return Vec::new();
    };
    if mapping_get(runs, "using").and_then(Value::as_str) != Some("composite") {
        return Vec::new();
    }
    let Some(steps) = mapping_get(runs, "steps").and_then(Value::as_sequence) else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        findings.extend(judge_call(
            rel,
            source,
            step,
            Site::Composite { index },
            opts,
        ));
    }
    findings
}

fn judge_call(
    rel: &str,
    source: &str,
    map: &Mapping,
    site: Site<'_>,
    opts: &Compiled,
) -> Vec<RuleFinding> {
    let Some(uses) = mapping_string(map, "uses") else {
        return Vec::new();
    };
    let Some(family) = policy::matching(&uses, &opts.actions) else {
        return Vec::new();
    };
    let label = step_label(map, &uses);
    let with = mapping_get(map, "with").and_then(Value::as_mapping);
    policy::judge(with, family)
        .into_iter()
        .map(|fault| {
            let key = message::fault_key(&fault);
            let (line, target) = place(source, site, key, &label);
            message::finding(
                rel,
                line,
                message::message(rel, site, &label, &family.name, &fault),
                &target,
            )
        })
        .collect()
}

fn place(source: &str, site: Site<'_>, key: &str, label: &str) -> (usize, String) {
    match site {
        Site::Step { job_id, index } => {
            (input_line(source, job_id, index, key), job_id.to_string())
        }
        Site::Composite { index } => (
            input_line(source, "(composite)", index, key),
            label.to_string(),
        ),
        Site::Job { job_id } => (job_input_line(source, job_id, key), job_id.to_string()),
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
