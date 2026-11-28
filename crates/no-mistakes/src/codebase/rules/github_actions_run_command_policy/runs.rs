use super::judge::{self, Hit, HitKind};
use super::options::CompiledOptions;
use super::RULE_ID;
use crate::codebase::rules::{
    github_actions_script_expressions::{step_run_lines, StepRun},
    github_actions_workflow_files::run_scripts::{run_scripts, RunScript},
    RuleFinding,
};
use crate::config::v2::Unanalyzable;
use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use serde_yaml::Value;

pub(super) fn check_parsed(
    rel: &str,
    source: &str,
    action: bool,
    value: &Value,
    compiled: &CompiledOptions,
    unanalyzable: Unanalyzable,
    rule_message: Option<&str>,
) -> AnalysisDiagnostics {
    let located = step_run_lines(source, action);
    let mut diagnostics = AnalysisDiagnostics::default();
    for body in run_scripts(action, value) {
        let step_lines = located
            .iter()
            .find(|item| item.job == body.job && item.step == body.step);
        let place = location_label(&body);
        for hit in judge::scan_script(&body.script, compiled) {
            let line = report_line(source, step_lines, &body.script, &hit);
            let finding = build_finding(rel, line, Some(&place), &hit, rule_message);
            if matches!(hit.kind, HitKind::Unanalyzable { .. }) {
                diagnostics.unanalyzable(unanalyzable, finding);
            } else {
                diagnostics.findings.push(finding);
            }
        }
    }
    diagnostics
}

pub(super) fn check_shell_source(
    rel: &str,
    source: &str,
    compiled: &CompiledOptions,
    unanalyzable: Unanalyzable,
    rule_message: Option<&str>,
) -> AnalysisDiagnostics {
    let source = source.trim_start_matches('\u{feff}');
    let mut diagnostics = AnalysisDiagnostics::default();
    for hit in judge::scan_script(source, compiled) {
        let line = hit.line.max(1);
        let finding = build_finding(rel, line, None, &hit, rule_message);
        if matches!(hit.kind, HitKind::Unanalyzable { .. }) {
            diagnostics.unanalyzable(unanalyzable, finding);
        } else {
            diagnostics.findings.push(finding);
        }
    }
    diagnostics
}

fn location_label(body: &RunScript) -> String {
    if body.composite {
        format!("composite step {}", body.step)
    } else {
        format!("job `{}` step {}", body.job, body.step)
    }
}

fn prefix(rel: &str, line: usize, place: Option<&str>) -> String {
    match place {
        Some(place) => format!("{rel}:{line}: {place}"),
        None => format!("{rel}:{line}:"),
    }
}

fn build_finding(
    rel: &str,
    line: usize,
    place: Option<&str>,
    hit: &Hit,
    rule_message: Option<&str>,
) -> RuleFinding {
    let head = prefix(rel, line, place);
    let (target, message) = match &hit.kind {
        HitKind::MissingRequired {
            command,
            required,
            policy_message,
        } => {
            let tail = custom_or_default(policy_message.as_deref().or(rule_message), || {
                format!("Add `{required}` to the `{command}` command.")
            });
            (
                command.clone(),
                format!("{head} runs `{command}` without required argument `{required}`. {tail}"),
            )
        }
        HitKind::MissingAnyRequired {
            command,
            alternatives,
            policy_message,
        } => {
            let alts = alternatives
                .iter()
                .map(|item| format!("`{item}`"))
                .collect::<Vec<_>>()
                .join(" or ");
            let tail = custom_or_default(policy_message.as_deref().or(rule_message), || {
                format!("Pass one of those arguments to `{command}`.")
            });
            (
                command.clone(),
                format!("{head} runs `{command}` without any of {alts}. {tail}"),
            )
        }
        HitKind::ForbiddenArg {
            command,
            argument,
            pattern,
            policy_message,
        } => {
            let tail = custom_or_default(policy_message.as_deref().or(rule_message), || {
                format!("Rewrite the `{command}` invocation without that argument pattern.")
            });
            (
                command.clone(),
                format!(
                    "{head} passes forbidden argument `{argument}` to `{command}` (matched `/{pattern}/`). {tail}"
                ),
            )
        }
        HitKind::Unanalyzable { detail, .. } => {
            ("unanalyzable".to_string(), format!("{head} {detail}"))
        }
    };
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message,
        import: None,
        target: Some(target),
    }
}

fn custom_or_default(custom: Option<&str>, fallback: impl FnOnce() -> String) -> String {
    match custom {
        Some(text) => text.to_string(),
        None => fallback(),
    }
}

fn report_line(source: &str, located: Option<&StepRun>, script: &str, hit: &Hit) -> usize {
    let Some(located) = located.filter(|item| !item.lines.is_empty()) else {
        return fallback_flow_line(source, hit);
    };
    if script.lines().count() == located.lines.len() {
        return located
            .lines
            .get(hit.line.saturating_sub(1))
            .map(|(line, _)| *line)
            .unwrap_or(located.lines[0].0);
    }
    if let Some(needle) = hit.token() {
        if let Some((line, _)) = located.lines.iter().find(|(_, text)| text.contains(needle)) {
            return *line;
        }
    }
    located.lines[0].0
}

fn fallback_flow_line(source: &str, hit: &Hit) -> usize {
    let Some(needle) = hit.token() else {
        return 1;
    };
    source
        .lines()
        .enumerate()
        .find(|(_, line)| line.contains(needle) && line.contains("run"))
        .map(|(index, _)| index + 1)
        .unwrap_or(1)
}
