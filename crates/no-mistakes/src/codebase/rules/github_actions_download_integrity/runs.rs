use super::judge::{self, Hit};
use super::{RuleFinding, SkipAnnotationScope, RULE_ID};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::github_actions_script_expressions::{step_run_lines, StepRun};
use crate::config::v2::Unanalyzable;
use serde_yaml::{Mapping, Value};

mod bodies;
use bodies::{bodies, Body};

mod findings;
use findings::{script_finding, workflow_finding, workflow_line};

mod annotations;
use annotations::{line_scope_attributable, step_skipped, yaml_line_skipped};

pub(super) struct Policy<'a> {
    pub hosts: &'a [String],
    pub helpers: &'a [String],
    pub low: Option<Unanalyzable>,
    pub strict: bool,
    pub skip_scope: SkipAnnotationScope,
}

impl Policy<'_> {
    fn hits(
        &self,
        script: &str,
        env: Option<&Mapping>,
        skip_scope: Option<SkipAnnotationScope>,
    ) -> Vec<Hit> {
        let hits = judge::scan_script(
            script,
            self.helpers,
            self.low.is_some(),
            self.strict,
            env,
            skip_scope,
        );
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
    value: &Value,
    action: bool,
    policy: &Policy<'_>,
) -> AnalysisDiagnostics {
    let source = source.trim_start_matches('\u{feff}');
    let located = step_run_lines(source, action);
    let lines: Vec<&str> = source.lines().collect();
    let mut diagnostics = AnalysisDiagnostics::default();
    for body in bodies(action, value) {
        let place = located
            .iter()
            .find(|item| item.job == body.job && item.step == body.step);
        if policy.skip_scope == SkipAnnotationScope::Step
            && step_skipped(&lines, place, body.script)
        {
            continue;
        }
        let skip_scope = Some(policy.skip_scope).filter(|scope| {
            *scope == SkipAnnotationScope::Step
                || line_scope_attributable(&lines, place, body.script)
        });
        for hit in policy.hits(body.script, body.env, skip_scope) {
            if skip_scope == Some(SkipAnnotationScope::Line)
                && yaml_line_skipped(&lines, place, workflow_line(place, hit.line))
            {
                continue;
            }
            let finding = workflow_finding(rel, &body, place, &hit, policy.strict);
            policy.record(&mut diagnostics, &hit, finding);
        }
    }
    diagnostics
}

pub(super) fn check_shell(rel: &str, source: &str, policy: &Policy<'_>) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    for hit in policy.hits(
        source.trim_start_matches('\u{feff}'),
        None,
        Some(policy.skip_scope),
    ) {
        let finding = script_finding(rel, &hit, policy.strict);
        policy.record(&mut diagnostics, &hit, finding);
    }
    diagnostics
}
