use super::yaml::{
    finding, is_direct_third_party, is_local_wrapper, key_line, literal_u64, mapping_get,
    mapping_string, nested_composite_message, nested_message, step_label, timeout_message,
    uses_matches, yaml_got,
};
use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use serde_yaml::{Mapping, Value};
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
        Err(_) => Vec::new(),
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
    if let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) {
        return check_workflow(rel, source, jobs, opts);
    }
    check_composite(rel, source, value, opts)
}

fn check_workflow(
    rel: &str,
    source: &str,
    jobs: &Mapping,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
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
            findings.extend(check_workflow_step(rel, source, job_id, step, index, opts));
        }
    }
    findings
}

fn check_workflow_step(
    rel: &str,
    source: &str,
    job_id: &str,
    step: &Mapping,
    index: usize,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    let Some(uses) = mapping_string(step, "uses") else {
        return Vec::new();
    };
    if !uses_matches(&uses, &opts.uses) {
        return Vec::new();
    }
    let label = step_label(step, index);
    let mut findings = Vec::new();
    if let Some(expected) = opts.step_timeout_minutes {
        let raw = mapping_get(step, "timeout-minutes");
        if raw.and_then(literal_u64) != Some(expected) {
            let key = if raw.is_some() {
                "timeout-minutes"
            } else {
                "uses"
            };
            findings.push(finding(
                rel,
                key_line(source, key),
                timeout_message(rel, job_id, &label, expected, &yaml_got(raw)),
                job_id,
            ));
        }
    }
    if is_direct_third_party(&uses, &opts.uses) {
        findings.extend(nested_input_finding(
            rel, source, job_id, &label, step, opts,
        ));
    }
    findings
}

fn check_composite(
    rel: &str,
    source: &str,
    value: &Value,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    let Some(runs) = value.get("runs").and_then(Value::as_mapping) else {
        return Vec::new();
    };
    if mapping_get(runs, "using").and_then(Value::as_str) != Some("composite") {
        return Vec::new();
    }
    let Some(steps) = mapping_get(runs, "steps").and_then(Value::as_sequence) else {
        return Vec::new();
    };
    let wrapper = is_local_wrapper(rel, &opts.uses);
    let mut findings = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        let Some(uses) = mapping_string(step, "uses") else {
            continue;
        };
        let label = step_label(step, index);
        if wrapper {
            if is_direct_third_party(&uses, &opts.uses) {
                findings.extend(nested_input_finding(
                    rel,
                    source,
                    "(composite)",
                    &label,
                    step,
                    opts,
                ));
            }
            continue;
        }
        if opts.forbid_nested_in_composite && uses_matches(&uses, &opts.uses) {
            findings.push(finding(
                rel,
                key_line(source, "uses"),
                nested_composite_message(rel, &label),
                &label,
            ));
        }
    }
    findings
}

fn nested_input_finding(
    rel: &str,
    source: &str,
    job_id: &str,
    label: &str,
    step: &Mapping,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    if opts.nested_input.is_empty() {
        return Vec::new();
    }
    let Some(expected) = opts.nested_timeout_seconds else {
        return Vec::new();
    };
    let raw = mapping_get(step, "with")
        .and_then(Value::as_mapping)
        .and_then(|with| mapping_get(with, &opts.nested_input));
    if raw.and_then(literal_u64) == Some(expected) {
        return Vec::new();
    }
    vec![finding(
        rel,
        key_line(source, &opts.nested_input),
        nested_message(
            rel,
            job_id,
            label,
            &opts.nested_input,
            expected,
            &yaml_got(raw),
        ),
        job_id,
    )]
}
