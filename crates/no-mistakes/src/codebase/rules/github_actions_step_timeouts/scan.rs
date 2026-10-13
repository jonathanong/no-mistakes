use super::yaml::{finding, has_key, is_expression, mapping_str, step_line, timeout_minutes};
use super::{CompiledOptions, RuleFinding, GITHUB_MAX_MINUTES, RULE_ID};
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
    if crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID) {
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
    super::super::suppress_rule_findings_with_source(&mut findings, &source);
    findings
}

fn check_parsed(
    rel: &str,
    source: &str,
    value: &Value,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    let mut used = BTreeSet::new();
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
        for (index, step) in steps.iter().enumerate() {
            let Some(step) = step.as_mapping() else {
                continue;
            };
            if !has_key(step, "run") && !has_key(step, "uses") {
                continue;
            }
            let step_id = mapping_str(step, "id")
                .map(str::to_owned)
                .unwrap_or_else(|| (index + 1).to_string());
            let key = format!("{rel}#{job_id}#{step_id}");
            let line = step_line(source, job_id, index).unwrap_or(1);
            let label = mapping_str(step, "name")
                .or_else(|| mapping_str(step, "id"))
                .map(str::to_owned)
                .unwrap_or_else(|| format!("step #{}", index + 1));
            let allowed = opts.allow.get(&key);
            if allowed.is_some() {
                used.insert(key.clone());
            }
            if allowed.is_some_and(|entry| entry.reason.trim().is_empty()) {
                findings.push(finding(
                    rel,
                    line,
                    format!("{rel}: allow entry `{key}` needs a reason"),
                    Some(&key),
                ));
            }
            let raw = step.get("timeout-minutes");
            let minutes = raw.and_then(timeout_minutes);
            match (raw, minutes, allowed) {
                (None, _, None) => findings.push(finding(rel, line,
                    format!("{rel}: job \"{job_id}\" step \"{label}\" has no timeout-minutes; add a positive literal number"), Some(&key))),
                (Some(raw), None, Some(_)) if is_expression(raw) => {}
                (Some(_), None, _) => findings.push(finding(rel, line,
                    format!("{rel}: job \"{job_id}\" step \"{label}\" timeout-minutes must be a positive literal number (expressions need a reasoned allow entry)"), Some(&key))),
                (Some(_), Some(minutes), allowed) => {
                    let cap = allowed.and_then(|entry| entry.max_minutes).unwrap_or(opts.max_minutes).min(GITHUB_MAX_MINUTES);
                    if minutes > cap {
                        findings.push(finding(rel, line,
                            format!("{rel}: job \"{job_id}\" step \"{label}\" timeout-minutes is {minutes}, over the {cap}-minute cap"), Some(&key)));
                    }
                }
                _ => {}
            }
        }
    }
    for key in opts.allow.keys() {
        if key.starts_with(&format!("{rel}#")) && !used.contains(key) {
            findings.push(finding(
                rel,
                1,
                format!("{rel}: stale {RULE_ID} allow entry `{key}`"),
                Some(key),
            ));
        }
    }
    for key in &opts.duplicate_allow {
        if key.starts_with(&format!("{rel}#")) {
            findings.push(finding(
                rel,
                1,
                format!("{rel}: duplicate {RULE_ID} allow entry `{key}`"),
                Some(key),
            ));
        }
    }
    findings
}
