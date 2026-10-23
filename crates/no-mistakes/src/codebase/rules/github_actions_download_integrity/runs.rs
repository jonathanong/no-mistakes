use super::judge::{self, Hit};
use super::{RuleFinding, RULE_ID};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::github_actions_script_expressions::{step_run_lines, StepRun};
use crate::config::v2::Unanalyzable;
use serde_yaml::{Mapping, Value};

mod findings;
use findings::finding;

struct Body {
    job: String,
    step: usize,
    script: String,
    target: String,
}

pub(super) fn check_source(
    rel: &str,
    source: &str,
    hosts: &[String],
    helpers: &[String],
) -> Vec<RuleFinding> {
    let Some(action) = super::gha_kind(rel) else {
        return Vec::new();
    };
    let source = source.trim_start_matches('\u{feff}');
    let Ok(value) = serde_yaml::from_str::<Value>(source) else {
        return Vec::new();
    };
    let located = step_run_lines(source, action);
    let lines: Vec<&str> = source.lines().collect();
    let mut findings = Vec::new();
    for body in bodies(action, &value) {
        let place = located
            .iter()
            .find(|item| item.job == body.job && item.step == body.step);
        if skipped(&lines, place, &body.script) {
            continue;
        }
        let hits = judge::filter_trusted(judge::download_hits(&body.script, helpers), hosts);
        findings.extend(hits.into_iter().map(|hit| finding(rel, &body, place, hit)));
    }
    findings
}

pub(super) fn check_source_low(
    rel: &str,
    source: &str,
    hosts: &[String],
    helpers: &[String],
    policy: Unanalyzable,
) -> AnalysisDiagnostics {
    let Some(action) = super::gha_kind(rel) else {
        return AnalysisDiagnostics::default();
    };
    let source = source.trim_start_matches('\u{feff}');
    let Ok(value) = serde_yaml::from_str::<Value>(source) else {
        return AnalysisDiagnostics::default();
    };
    let located = step_run_lines(source, action);
    let lines: Vec<&str> = source.lines().collect();
    let mut diagnostics = AnalysisDiagnostics::default();
    for body in bodies(action, &value) {
        let place = located
            .iter()
            .find(|item| item.job == body.job && item.step == body.step);
        if skipped(&lines, place, &body.script) {
            continue;
        }
        let hits = judge::filter_trusted(judge::download_hits_low(&body.script, helpers), hosts);
        for hit in hits {
            let unanalyzable = hit.unanalyzable;
            let finding = finding(rel, &body, place, hit);
            if unanalyzable {
                diagnostics.unanalyzable(policy, finding);
            } else {
                diagnostics.findings.push(finding);
            }
        }
    }
    diagnostics
}

fn skipped(lines: &[&str], place: Option<&StepRun>, script: &str) -> bool {
    if judge::has_skip(script) {
        return true;
    }
    let Some(place) = place else {
        return false;
    };
    // The skip comment sits above the `- ` item, which may be above `run:`.
    let start = place.item_line;
    if start >= 2 {
        let prev = lines.get(start - 2).copied().unwrap_or("");
        if prev.trim_start().starts_with('#') && judge::has_skip(prev) {
            return true;
        }
    }
    let last = place.lines.last().map(|(line, _)| *line).unwrap_or(start);
    lines
        .iter()
        .take(last)
        .skip(start.saturating_sub(1))
        .any(|line| judge::has_skip(line))
}

fn bodies(action: bool, value: &Value) -> Vec<Body> {
    if action {
        return composite_bodies(value);
    }
    let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (id, job) in jobs {
        let Some(job) = job.as_mapping() else {
            continue;
        };
        let Some(steps) = mapping_get(job, "steps").and_then(Value::as_sequence) else {
            continue;
        };
        push_runs(&scalar_name(id), steps, &mut out);
    }
    out
}

fn composite_bodies(value: &Value) -> Vec<Body> {
    if value
        .get("runs")
        .and_then(|runs| runs.get("using"))
        .and_then(Value::as_str)
        != Some("composite")
    {
        return Vec::new();
    }
    let Some(steps) = value
        .get("runs")
        .and_then(|runs| runs.get("steps"))
        .and_then(Value::as_sequence)
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    push_runs("", steps, &mut out);
    out
}

fn push_runs(job: &str, steps: &[Value], out: &mut Vec<Body>) {
    for (index, step) in steps.iter().enumerate() {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        let Some(script) = mapping_get(step, "run").and_then(Value::as_str) else {
            continue;
        };
        let target = if job.is_empty() {
            "composite".to_string()
        } else {
            job.to_string()
        };
        out.push(Body {
            job: job.to_string(),
            step: index + 1,
            script: script.to_string(),
            target,
        });
    }
}

fn mapping_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_owned()))
}

fn scalar_name(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => "job".to_string(),
    }
}
