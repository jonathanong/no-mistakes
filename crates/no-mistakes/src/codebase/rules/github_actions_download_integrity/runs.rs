use super::judge::{self, Hit};
use super::{RuleFinding, RULE_ID};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::github_actions_script_expressions::{step_run_lines, StepRun};
use crate::config::v2::Unanalyzable;
use serde_yaml::{Mapping, Value};

mod bodies;
use bodies::{bodies, Body};

mod findings;
use findings::{script_finding, workflow_finding};

pub(super) struct Policy<'a> {
    pub hosts: &'a [String],
    pub helpers: &'a [String],
    pub low: Option<Unanalyzable>,
    pub strict: bool,
}

impl Policy<'_> {
    fn hits(&self, script: &str, env: Option<&Mapping>) -> Vec<Hit> {
        let hits = judge::scan_script(script, self.helpers, self.low.is_some(), self.strict, env);
        judge::filter_trusted(hits, self.hosts)
    }

    fn record(&self, diagnostics: &mut AnalysisDiagnostics, hit: &Hit, finding: RuleFinding) {
        if let Some(policy) = self.low.filter(|_| hit.unanalyzable) {
            diagnostics.unanalyzable(policy, finding);
        } else {
            diagnostics.findings.push(finding);
        }
    }
}

pub(super) fn check_workflow(
    rel: &str,
    source: &str,
    action: bool,
    policy: &Policy<'_>,
) -> AnalysisDiagnostics {
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
        if skipped(&lines, place, body.script) {
            continue;
        }
        for hit in policy.hits(body.script, body.env) {
            let finding = workflow_finding(rel, &body, place, &hit, policy.strict);
            policy.record(&mut diagnostics, &hit, finding);
        }
    }
    diagnostics
}

pub(super) fn check_shell(rel: &str, source: &str, policy: &Policy<'_>) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    for hit in policy.hits(source.trim_start_matches('\u{feff}'), None) {
        let finding = script_finding(rel, &hit, policy.strict);
        policy.record(&mut diagnostics, &hit, finding);
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
