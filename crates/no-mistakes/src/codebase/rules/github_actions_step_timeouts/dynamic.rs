use super::options::CompiledOptions;
use super::yaml::{finding, timeout_minutes};
use super::GITHUB_MAX_MINUTES;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use serde_yaml::Mapping;
use std::collections::BTreeSet;

pub(super) struct DynamicCtx<'a> {
    pub(super) rel: &'a str,
    pub(super) job_id: &'a str,
    pub(super) opts: &'a CompiledOptions,
}

pub(super) struct StepRef<'a> {
    pub(super) key: &'a str,
    pub(super) label: &'a str,
    pub(super) line: usize,
}

pub(super) fn check_dynamic_only(
    ctx: &DynamicCtx<'_>,
    info: StepRef<'_>,
    step: &Mapping,
    raw_uses: &str,
    diag: &mut AnalysisDiagnostics,
    used: &mut BTreeSet<String>,
) {
    if let Some(entry) = ctx.opts.allow.get(info.key) {
        if entry.reason.trim().is_empty() {
            diag.findings.push(finding(
                ctx.rel,
                info.line,
                format!("{}: allow entry `{}` needs a reason", ctx.rel, info.key),
                Some(info.key),
            ));
        }
        used.insert(info.key.to_string());
        if let Some(cap) = entry.max_minutes.map(|max| max.min(GITHUB_MAX_MINUTES)) {
            if let Some(minutes) = step.get("timeout-minutes").and_then(timeout_minutes) {
                if minutes > cap {
                    diag.findings.push(finding(
                        ctx.rel,
                        info.line,
                        format!(
                            "{}: job \"{}\" step \"{}\" timeout-minutes is {minutes}, over the {cap}-minute cap",
                            ctx.rel, ctx.job_id, info.label
                        ),
                        Some(info.key),
                    ));
                }
            }
        }
        return;
    }
    let minutes = step.get("timeout-minutes").and_then(timeout_minutes);
    if ctx
        .opts
        .min_action_cap
        .is_some_and(|min_cap| minutes.is_some_and(|value| value <= min_cap))
    {
        return;
    }
    emit_dynamic_uses(ctx, info, raw_uses, diag);
}

pub(super) fn emit_dynamic_uses(
    ctx: &DynamicCtx<'_>,
    info: StepRef<'_>,
    raw_uses: &str,
    diag: &mut AnalysisDiagnostics,
) {
    diag.unanalyzable(
        ctx.opts.unanalyzable,
        finding(
            ctx.rel,
            info.line,
            format!(
                "{}: job \"{}\" step \"{}\" uses dynamic action `{raw_uses}`, which cannot be statically matched against configured actionCaps; use a concrete action reference or set timeout-minutes within the strictest action cap",
                ctx.rel, ctx.job_id, info.label
            ),
            Some(info.key),
        ),
    );
}
