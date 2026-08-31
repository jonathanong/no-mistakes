use super::{RuleFinding, FORBIDDEN_KEYS, RULE_ID};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use serde_yaml::{Mapping, Value};
use std::path::Path;

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
                let line = checkout_key_line(source, key, occurrence);
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
        .is_some_and(|uses| uses.trim().starts_with("actions/checkout@"))
}

fn checkout_key_line(source: &str, key: &str, occurrence: usize) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let checkout_line = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains("uses:") && line.contains("actions/checkout@"))
        .nth(occurrence)
        .map(|(index, _)| index)
        .unwrap_or(0);
    let step_indent = leading_indent(lines[checkout_line]);
    let mut with_indent = None;
    for (index, line) in lines.iter().enumerate().skip(checkout_line + 1) {
        let indent = leading_indent(line);
        let trimmed = line.trim();
        if trimmed.starts_with("- ") && indent <= step_indent {
            break;
        }
        if with_indent.is_none() && trimmed.starts_with("with:") {
            with_indent = Some(indent);
            continue;
        }
        if let Some(with_indent) = with_indent {
            if !trimmed.is_empty() && indent <= with_indent {
                break;
            }
            if yaml_key_line(trimmed, key) {
                return index + 1;
            }
        }
    }
    checkout_line + 1
}

fn leading_indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn yaml_key_line(line: &str, key: &str) -> bool {
    line.starts_with(&format!("{key}:"))
        || line.starts_with(&format!("'{key}':"))
        || line.starts_with(&format!("\"{key}\":"))
}
