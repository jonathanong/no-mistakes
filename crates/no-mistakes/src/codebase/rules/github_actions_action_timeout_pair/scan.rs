use super::line::{nested_input_finding, nested_input_value, step_key_line};
use super::workflow_step::check_workflow_step;
use super::yaml::{
    finding, is_direct_third_party, is_local_wrapper, mapping_get, mapping_string,
    nested_composite_message, step_label, uses_matches, wrapper_rel,
};
use super::{AnalysisDiagnostics, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use serde_yaml::{Mapping, Value};
use std::path::Path;

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> AnalysisDiagnostics {
    let rel = relative_slash_path(root, path);
    let Some(source) = super::super::read_language_source(sources, path) else {
        return AnalysisDiagnostics::default();
    };
    if !defer_suppression && crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
    {
        return AnalysisDiagnostics::default();
    }
    let wrapper_rel = wrapper_rel(root, path, &opts.target_roots);
    let mut diagnostics = match serde_yaml::from_str::<Value>(&source) {
        Ok(value) => check_parsed(&rel, &wrapper_rel, &source, &value, opts),
        Err(_) => AnalysisDiagnostics::default(),
    };
    if !defer_suppression {
        super::super::suppress_rule_findings_for_path(sources, path, &mut diagnostics.findings);
        super::super::suppress_rule_findings_for_path(sources, path, &mut diagnostics.advisories);
    }
    diagnostics
}

pub(super) fn check_parsed(
    rel: &str,
    wrapper_rel: &str,
    source: &str,
    value: &Value,
    opts: &CompiledOptions,
) -> AnalysisDiagnostics {
    if let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) {
        return check_workflow(rel, source, jobs, opts);
    }
    AnalysisDiagnostics {
        findings: check_composite(rel, wrapper_rel, source, value, opts),
        ..Default::default()
    }
}

fn check_workflow(
    rel: &str,
    source: &str,
    jobs: &Mapping,
    opts: &CompiledOptions,
) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
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
            check_workflow_step(rel, source, job_id, step, index, opts, &mut diagnostics);
        }
    }
    diagnostics
}

fn check_composite(
    rel: &str,
    wrapper_rel: &str,
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
    let wrapper = is_local_wrapper(wrapper_rel, &opts.uses);
    let mut findings = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        let Some(uses) = mapping_string(step, "uses") else {
            continue;
        };
        let label = step_label(step, index);
        if !wrapper && opts.forbid_nested_in_composite && uses_matches(&uses, &opts.uses) {
            findings.push(finding(
                rel,
                step_key_line(source, "(composite)", index, "uses"),
                nested_composite_message(rel, &label),
                &label,
            ));
            continue;
        }
        if is_direct_third_party(&uses, &opts.uses) {
            findings.extend(nested_input_finding(
                rel,
                source,
                "(composite)",
                &label,
                nested_input_value(step, &opts.nested_input),
                index,
                opts,
            ));
        }
    }
    findings
}
