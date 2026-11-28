use super::scripts::{ScriptHit, ScriptHitKind};
use super::triggers::{EventFilter, FilterKind};
use super::RULE_ID;
use crate::codebase::rules::{
    analysis_policy::AnalysisDiagnostics, github_actions_script_expressions::StepRun, RuleFinding,
};
use crate::config::v2::Unanalyzable;
use std::collections::BTreeSet;

pub(super) struct EmitCtx<'a> {
    pub(super) rel: &'a str,
    pub(super) source: &'a str,
    pub(super) filters: &'a [EventFilter],
    pub(super) unanalyzable: Unanalyzable,
    pub(super) rule_message: Option<&'a str>,
}

pub(super) enum Origin<'a> {
    Run {
        place: &'a str,
        located: Option<&'a StepRun>,
        script: &'a str,
    },
    Composite {
        place: &'a str,
        action: &'a str,
        uses_line: usize,
    },
}

pub(super) fn emit_hits(
    ctx: &EmitCtx<'_>,
    origin: Origin<'_>,
    hits: &[ScriptHit],
    diag: &mut AnalysisDiagnostics,
) {
    let mut seen_scripts = BTreeSet::new();
    for hit in hits {
        let line = match origin {
            Origin::Composite { uses_line, .. } => uses_line,
            Origin::Run {
                located, script, ..
            } => report_line(ctx.source, located, script, hit),
        };
        let label = match origin {
            Origin::Composite { place, action, .. } => format!("{place} (via `{action}`)"),
            Origin::Run { place, .. } => place.to_string(),
        };
        match &hit.kind {
            ScriptHitKind::Invocation { path, .. } => {
                if !seen_scripts.insert((line, path.clone())) {
                    continue;
                }
                let missed: Vec<&EventFilter> = ctx
                    .filters
                    .iter()
                    .filter(|filter| !filter.covers(path))
                    .collect();
                if missed.is_empty() {
                    continue;
                }
                let events_text = missed
                    .iter()
                    .map(|filter| filter.label())
                    .collect::<Vec<_>>()
                    .join(", ");
                let tail = ctx
                    .rule_message
                    .map(str::trim)
                    .filter(|msg| !msg.is_empty())
                    .map_or_else(
                        || remediation_hint(path, &missed, &events_text),
                        str::to_string,
                    );
                diag.findings.push(finding(
                    ctx.rel,
                    line,
                    format!(
                        "{}:{line}: {label} runs repository script `{path}`, which is not covered by {events_text}. {tail}",
                        ctx.rel
                    ),
                    path,
                ));
            }
            ScriptHitKind::Unanalyzable { detail, .. } => {
                diag.unanalyzable(
                    ctx.unanalyzable,
                    finding(
                        ctx.rel,
                        line,
                        format!("{}:{line}: {label} {detail}", ctx.rel),
                        "unanalyzable",
                    ),
                );
            }
        }
    }
}

pub(super) fn finding(file: &str, line: usize, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}

fn remediation_hint(path: &str, missed: &[&EventFilter], events_text: &str) -> String {
    let all_paths = missed.iter().all(|f| matches!(f.kind, FilterKind::Paths));
    let all_ignore = missed
        .iter()
        .all(|f| matches!(f.kind, FilterKind::PathsIgnore));
    if all_paths {
        format!("Add `{path}` (or its script directory) to {events_text} so edits to this script trigger the workflow.")
    } else if all_ignore {
        format!("Remove `{path}` from {events_text}, or narrow that ignore pattern. Use `paths` when the workflow should include this script and exclude others.")
    } else {
        format!("Update {events_text} so edits to `{path}` trigger the workflow.")
    }
}

fn report_line(source: &str, located: Option<&StepRun>, script: &str, hit: &ScriptHit) -> usize {
    let Some(located) = located.filter(|item| !item.lines.is_empty()) else {
        return hit
            .token()
            .and_then(|needle| {
                source
                    .lines()
                    .enumerate()
                    .find(|(_, line)| line.contains(needle) && line.contains("run"))
                    .map(|(i, _)| i + 1)
            })
            .unwrap_or(1);
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
