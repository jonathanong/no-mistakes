use super::{RuleFinding, FORBIDDEN_KEYS, RULE_ID};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use serde_yaml::{Mapping, Value};
use std::path::Path;

mod location;

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    sources: &SourceStore,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    let Some(source) = super::super::read_source(sources, path) else {
        return Vec::new();
    };
    let file = relative_slash_path(root, path);
    if !defer_suppression && crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
    {
        return Vec::new();
    }
    let mut findings = match serde_yaml::from_str::<Value>(&source) {
        Ok(value) => check_document(&file, &source, &value),
        Err(error) => vec![RuleFinding {
            rule: RULE_ID.to_string(),
            file: file.clone(),
            line: error.location().map_or(1, |location| location.line()),
            message: format!("{file}: invalid YAML ({error})"),
            import: None,
            target: None,
        }],
    };
    if !defer_suppression {
        super::super::suppress_rule_findings_with_source(&mut findings, &source);
    }
    findings
}

fn check_document(file: &str, source: &str, value: &Value) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    let mut checkout_occurrence = 0;
    if let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) {
        for (job_name, job) in jobs {
            let Some(job) = job.as_mapping() else {
                continue;
            };
            let label = job_name.as_str().unwrap_or("unnamed job");
            findings.extend(check_steps(
                file,
                source,
                job.get("steps").and_then(Value::as_sequence),
                &format!("job \"{label}\""),
                &mut checkout_occurrence,
            ));
        }
    }
    if let Some(runs) = value.get("runs").and_then(Value::as_mapping) {
        findings.extend(check_steps(
            file,
            source,
            runs.get("steps").and_then(Value::as_sequence),
            "composite action",
            &mut checkout_occurrence,
        ));
    }
    findings
}

fn check_steps(
    file: &str,
    source: &str,
    steps: Option<&Vec<Value>>,
    owner: &str,
    checkout_occurrence: &mut usize,
) -> Vec<RuleFinding> {
    let Some(steps) = steps else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    for step in steps.iter().filter_map(Value::as_mapping) {
        if !is_checkout(step) {
            continue;
        }
        let occurrence = *checkout_occurrence;
        *checkout_occurrence += 1;
        let Some(with) = step.get("with").and_then(Value::as_mapping) else {
            continue;
        };
        for key in FORBIDDEN_KEYS {
            if with.contains_key(Value::String((*key).to_string())) {
                let line = location::checkout_key_line(source, key, occurrence);
                findings.push(RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: file.to_string(),
                    line,
                    message: format!(
                        "{file}:{line}: {owner} passes `{key}` to actions/checkout; remove partial-checkout inputs so CI sees the full repository"
                    ),
                    import: None,
                    target: Some((*key).to_string()),
                });
            }
        }
    }
    findings
}

fn is_checkout(step: &Mapping) -> bool {
    step.get("uses")
        .and_then(Value::as_str)
        .is_some_and(location::is_checkout_reference)
}

#[cfg(test)]
mod tests;
