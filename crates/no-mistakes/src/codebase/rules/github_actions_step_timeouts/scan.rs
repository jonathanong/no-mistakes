use super::options::CompiledOptions;
use super::steps::scan_steps;
use super::yaml::{finding, StepLocations};
use super::RULE_ID;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
) -> AnalysisDiagnostics {
    let rel = crate::codebase::ts_source::relative_slash_path(root, path);
    let Some(source) = super::super::read_language_source(sources, path) else {
        return AnalysisDiagnostics::default();
    };
    if !opts.defer_suppression
        && crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
    {
        return AnalysisDiagnostics::default();
    }
    let mut diag = match sources.parse_loaded_yaml(path, &source, false) {
        Ok(value) => check_parsed(&rel, &source, &value, opts),
        Err(err) => {
            let mut out = AnalysisDiagnostics::default();
            out.findings.push(finding(
                &rel,
                err.line(),
                format!("{rel}: invalid YAML ({err})"),
                None,
            ));
            out
        }
    };
    for key in opts.duplicate_by_file.get(&rel).into_iter().flatten() {
        diag.findings.push(finding(
            &rel,
            1,
            format!("{rel}: duplicate {RULE_ID} allow entry `{key}`"),
            Some(key),
        ));
    }
    if !opts.defer_suppression {
        super::super::suppress_rule_findings_for_path(sources, path, &mut diag.findings);
        super::super::suppress_rule_findings_for_path(sources, path, &mut diag.advisories);
    }
    diag
}

fn check_parsed(
    rel: &str,
    source: &str,
    value: &Value,
    opts: &CompiledOptions,
) -> AnalysisDiagnostics {
    let jobs = value.get("jobs").and_then(Value::as_mapping);
    let mut diag = AnalysisDiagnostics::default();
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
            scan_steps(rel, job_id, steps, opts, &locations, &mut diag, &mut used);
        }
    }
    for key in opts.allow_by_file.get(rel).into_iter().flatten() {
        if !used.contains(key) {
            diag.findings.push(finding(
                rel,
                1,
                format!("{rel}: stale {RULE_ID} allow entry `{key}`"),
                Some(key),
            ));
        }
    }
    diag
}
