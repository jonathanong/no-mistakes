use super::line::{nested_input_finding, nested_input_value, step_key_line};
use super::workflow_step::check_workflow_step;
use super::yaml::{
    finding, is_direct_third_party, is_local_wrapper, mapping_get, mapping_string,
    nested_composite_message, step_label, uses_matches, wrapper_rel,
};
use super::{AnalysisDiagnostics, CompiledOptions, RULE_ID};
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use rayon::prelude::*;
use serde_yaml::{Mapping, Value};
use std::path::{Path, PathBuf};

pub(super) fn scan_files(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> (AnalysisDiagnostics, Vec<RuleFinding>) {
    let mut diagnostics = AnalysisDiagnostics::default();
    let mut input_failures = Vec::new();
    let chunks: Vec<_> = files
        .par_iter()
        .map(|path| check_file(root, path, opts, sources, defer_suppression))
        .collect();
    for chunk in chunks {
        match chunk {
            Ok(per_file) => diagnostics.append(per_file),
            Err(finding) => input_failures.push(*finding),
        }
    }
    (diagnostics, input_failures)
}

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics, Box<RuleFinding>> {
    let rel = relative_slash_path(root, path);
    let Some(source) = super::super::read_language_source(sources, path) else {
        return Ok(AnalysisDiagnostics::default());
    };
    if !defer_suppression && crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
    {
        return Ok(AnalysisDiagnostics::default());
    }
    let wrapper_rel = wrapper_rel(root, path, &opts.target_roots);
    let value = super::super::github_actions_workflow_files::parse_yaml(
        root, path, sources, &source, RULE_ID, false,
    )?;
    let mut diagnostics = check_parsed(&rel, &wrapper_rel, &source, &value, opts);
    if !defer_suppression {
        super::super::suppress_rule_findings_for_path(sources, path, &mut diagnostics.findings);
        super::super::suppress_rule_findings_for_path(sources, path, &mut diagnostics.advisories);
    }
    Ok(diagnostics)
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
