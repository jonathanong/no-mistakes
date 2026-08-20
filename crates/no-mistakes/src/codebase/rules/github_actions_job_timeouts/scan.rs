use super::yaml::{finding, key_line, step_label, timeout_minutes};
use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    let Some(source) = super::super::read_source(sources, path) else {
        return Vec::new();
    };
    if crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID) {
        return Vec::new();
    }
    let mut findings = match serde_yaml::from_str::<Value>(&source) {
        Ok(value) => check_parsed(&rel, &source, &value, opts),
        Err(err) => vec![RuleFinding {
            rule: RULE_ID.to_string(),
            file: rel.clone(),
            line: err.location().map_or(1, |location| location.line()),
            message: format!("{rel}: invalid YAML ({err})"),
            import: None,
            target: None,
        }],
    };
    super::super::suppress_rule_findings_with_source(&mut findings, &source);
    findings
}

pub(super) fn check_parsed(
    rel: &str,
    source: &str,
    value: &Value,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    let mut used_allow = BTreeSet::new();
    for (key, job) in jobs {
        let Some(job_id) = key.as_str() else {
            continue;
        };
        let Some(job) = job.as_mapping() else {
            continue;
        };
        if job.contains_key(Value::String("uses".to_string())) {
            continue;
        }
        findings.extend(check_job(rel, source, job_id, job, opts, &mut used_allow));
    }
    for allowed in opts.allow.keys() {
        if used_allow.contains(allowed) || !allowed.starts_with(&format!("{rel}#")) {
            continue;
        }
        findings.push(finding(
            rel,
            1,
            format!("stale github-actions-job-timeouts allow entry `{allowed}`"),
            allowed,
        ));
    }
    findings
}

fn check_job(
    rel: &str,
    source: &str,
    job_id: &str,
    job: &serde_yaml::Mapping,
    opts: &CompiledOptions,
    used_allow: &mut BTreeSet<String>,
) -> Vec<RuleFinding> {
    let allow_key = format!("{rel}#{job_id}");
    let line = key_line(source, job_id);
    let Some(raw) = job.get(Value::String("timeout-minutes".to_string())) else {
        return vec![finding(
            rel,
            line,
            format!("{rel}: job \"{job_id}\" has no timeout-minutes (defaults to GitHub's 6-hour ceiling); add a literal number"),
            job_id,
        )];
    };
    let Some(minutes) = timeout_minutes(raw) else {
        return vec![finding(
            rel,
            line,
            format!("{rel}: job \"{job_id}\" timeout-minutes has no supported literal upper bound"),
            job_id,
        )];
    };
    let mut findings = cap_findings(rel, line, job_id, minutes, &allow_key, opts, used_allow);
    if opts.reject_step_exceeding_job {
        findings.extend(step_findings(rel, source, job_id, job, minutes));
    }
    findings
}

fn cap_findings(
    rel: &str,
    line: usize,
    job_id: &str,
    minutes: u64,
    allow_key: &str,
    opts: &CompiledOptions,
    used_allow: &mut BTreeSet<String>,
) -> Vec<RuleFinding> {
    let Some(cap) = opts.max_minutes else {
        return Vec::new();
    };
    if minutes <= cap {
        return Vec::new();
    }
    if let Some(allowed) = opts.allow.get(allow_key) {
        used_allow.insert(allow_key.to_string());
        if allowed.is_none_or(|max| minutes <= max) {
            return Vec::new();
        }
        return vec![finding(
            rel,
            line,
            format!(
                "{rel}: job \"{job_id}\" timeout-minutes is {minutes}, exceeding its allowlisted max of {}",
                allowed.unwrap()
            ),
            job_id,
        )];
    }
    vec![finding(
        rel,
        line,
        format!("{rel}: job \"{job_id}\" timeout-minutes is {minutes}, over the {cap}-minute cap"),
        job_id,
    )]
}

fn step_findings(
    rel: &str,
    source: &str,
    job_id: &str,
    job: &serde_yaml::Mapping,
    job_timeout: u64,
) -> Vec<RuleFinding> {
    let Some(steps) = job
        .get(Value::String("steps".to_string()))
        .and_then(Value::as_sequence)
    else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        let Some(raw) = step.get(Value::String("timeout-minutes".to_string())) else {
            continue;
        };
        let label = step_label(step, index);
        let Some(minutes) = timeout_minutes(raw) else {
            findings.push(finding(
                rel,
                key_line(source, "timeout-minutes"),
                format!(
                    "{rel}: job \"{job_id}\" step \"{label}\" timeout-minutes has no supported literal upper bound"
                ),
                job_id,
            ));
            continue;
        };
        if minutes > job_timeout {
            findings.push(finding(
                rel,
                key_line(source, "timeout-minutes"),
                format!(
                    "{rel}: job \"{job_id}\" step \"{label}\" timeout-minutes is {minutes}, exceeding its job timeout of {job_timeout}"
                ),
                job_id,
            ));
        }
    }
    findings
}
