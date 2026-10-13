use super::steps::scan_steps;
use super::yaml::{finding, StepLocations};
use super::{CompiledOptions, RuleFinding, RULE_ID};
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let rel = crate::codebase::ts_source::relative_slash_path(root, path);
    let Some(source) = super::super::read_source(sources, path) else {
        return Vec::new();
    };
    if !opts.defer_suppression
        && crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
    {
        return Vec::new();
    }
    let mut findings = match serde_yaml::from_str::<Value>(&source) {
        Ok(value) => check_parsed(&rel, &source, &value, opts),
        Err(err) => vec![finding(
            &rel,
            err.location().map_or(1, |loc| loc.line()),
            format!("{rel}: invalid YAML ({err})"),
            None,
        )],
    };
    for key in opts.duplicate_by_file.get(&rel).into_iter().flatten() {
        findings.push(finding(
            &rel,
            1,
            format!("{rel}: duplicate {RULE_ID} allow entry `{key}`"),
            Some(key),
        ));
    }
    if !opts.defer_suppression {
        super::super::suppress_rule_findings_with_source(&mut findings, &source);
    }
    findings
}

fn check_parsed(
    rel: &str,
    source: &str,
    value: &Value,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    let jobs = value.get("jobs").and_then(Value::as_mapping);
    let mut findings = Vec::new();
    let mut used = BTreeSet::new();
    let locations = StepLocations::new(source);
    if let Some(jobs) = jobs {
        for (job_id, job) in jobs {
            let (Some(job_id), Some(job)) = (job_id.as_str(), job.as_mapping()) else {
                continue;
            };
            if job.contains_key(Value::String("uses".into())) {
                continue;
            }
            let Some(steps) = job.get("steps").and_then(Value::as_sequence) else {
                continue;
            };
            scan_steps(
                rel,
                job_id,
                steps,
                opts,
                &locations,
                &mut findings,
                &mut used,
            );
        }
    }
    for key in opts.allow_by_file.get(rel).into_iter().flatten() {
        if !used.contains(key) {
            findings.push(finding(
                rel,
                1,
                format!("{rel}: stale {RULE_ID} allow entry `{key}`"),
                Some(key),
            ));
        }
    }
    findings
}
