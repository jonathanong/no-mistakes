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
        let cap = entry
            .max_minutes
            .map(|max| max.min(GITHUB_MAX_MINUTES))
            .unwrap_or_else(|| ctx.opts.max_minutes.min(GITHUB_MAX_MINUTES));
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
        return;
    }
    let minutes = step.get("timeout-minutes").and_then(timeout_minutes);
    let global_cap = ctx.opts.max_minutes.min(GITHUB_MAX_MINUTES);
    if let Some(minutes) = minutes {
        if minutes > global_cap {
            diag.findings.push(finding(
                ctx.rel,
                info.line,
                format!(
                    "{}: job \"{}\" step \"{}\" timeout-minutes is {minutes}, over the {global_cap}-minute cap",
                    ctx.rel, ctx.job_id, info.label
                ),
                Some(info.key),
            ));
            return;
        }
        if ctx
            .opts
            .min_action_cap
            .is_some_and(|min_cap| minutes <= min_cap)
        {
            return;
        }
    }
    emit_dynamic_uses(ctx, info, raw_uses, diag);
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests {
    use super::super::options::compile_options;
    use super::{check_dynamic_only, DynamicCtx, StepRef};
    use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
    use crate::config::v2::Unanalyzable;
    use serde_yaml::Mapping;
    use std::collections::BTreeSet;

    fn compiled(yaml: &str) -> super::super::options::CompiledOptions {
        compile_options(serde_yaml::from_str(yaml).unwrap(), Unanalyzable::Report, false).unwrap()
    }

    fn dynamic_findings(options: &str, step_yaml: &str) -> AnalysisDiagnostics {
        let opts = compiled(options);
        let step: Mapping = serde_yaml::from_str(step_yaml).unwrap();
        let mut diag = AnalysisDiagnostics::default();
        let mut used = BTreeSet::new();
        let ctx = DynamicCtx {
            rel: "ci.yml",
            job_id: "build",
            opts: &opts,
        };
        check_dynamic_only(
            &ctx,
            StepRef {
                key: "ci.yml#build#1",
                label: "setup",
                line: 4,
            },
            &step,
            "${{ inputs.action }}",
            &mut diag,
            &mut used,
        );
        diag
    }

    #[test]
    fn dynamic_timeout_over_max_minutes_still_reports_inside_the_smallest_cap() {
        let over = dynamic_findings(
            "maxMinutes: 3\nonlyConfiguredActions: true\nactionCaps:\n  - {uses: actions/setup-node, maxMinutes: 10}\n",
            "timeout-minutes: 5\nuses: ${{ inputs.action }}\n",
        );
        let over_messages: Vec<_> = over.findings.iter().map(|f| f.message.as_str()).collect();
        assert!(
            over_messages.iter().any(|message| message.contains("over the 3-minute cap")),
            "{over_messages:?}"
        );
        assert!(over.advisories.is_empty(), "{}", over.advisories.len());

        let within = dynamic_findings(
            "maxMinutes: 10\nonlyConfiguredActions: true\nactionCaps:\n  - {uses: actions/setup-node, maxMinutes: 10}\n",
            "timeout-minutes: 5\nuses: ${{ inputs.action }}\n",
        );
        let within_messages: Vec<_> = within.findings.iter().map(|f| f.message.as_str()).collect();
        assert!(within_messages.is_empty(), "{within_messages:?}");

        let above_min = dynamic_findings(
            "maxMinutes: 10\nonlyConfiguredActions: true\nactionCaps:\n  - {uses: actions/setup-node, maxMinutes: 4}\n",
            "timeout-minutes: 5\nuses: ${{ inputs.action }}\n",
        );
        let above_messages: Vec<_> = above_min
            .findings
            .iter()
            .map(|finding| finding.message.as_str())
            .collect();
        assert!(
            above_messages
                .iter()
                .any(|message| message.contains("dynamic action")),
            "{above_messages:?}"
        );
    }

    #[test]
    fn wildcard_action_families_fail_at_config_load() {
        let error = compile_options(
            serde_yaml::from_str("actionCaps: [{uses: 'actions/*', maxMinutes: 5}]").unwrap(),
            Unanalyzable::Advise,
            false,
        )
        .map(drop)
        .unwrap_err();
        assert!(
            error.to_string().contains("concrete action family"),
            "{error}"
        );
    }
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
