use crate::check_parallel::DomainResults;
use anyhow::Result;
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::unique_exports::UniqueExportFinding;
use no_mistakes::config::v2::DisableDirectivePolicy;
use no_mistakes::integration_tests::IntegrationFinding;
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;
use std::time::Duration;
mod advisories;
mod partial;
pub(crate) use partial::retain_domain_checks;
mod empty;
pub(crate) use empty::{empty_results, CompletedDomainChecks};
mod render;
pub(crate) use render::json_value;
pub(crate) mod suppression;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod suppression_tests;

pub(crate) struct FinalizeInput<'a> {
    pub(crate) root: &'a std::path::Path,
    pub(crate) facts: &'a no_mistakes::codebase::check_facts::CheckFactMap,
    pub(crate) runner_config_deadlines:
        Option<Vec<no_mistakes::integration_tests::RunnerConfigDeadlineEvidence>>,
    pub(crate) config: &'a no_mistakes::config::v2::NoMistakesConfig,
    pub(crate) disable_directives:
        &'a no_mistakes::codebase::rules::disable_directive_audit::PreparedDisableDirectives,
    pub(crate) filesystem_files: &'a [std::path::PathBuf],
    pub(crate) sources: &'a no_mistakes::codebase::ts_source::SourceStore,
    pub(crate) filesystem_rules_enabled: bool,
    pub(crate) react_warning: Option<String>,
    pub(crate) discover_duration: Duration,
    pub(crate) facts_duration: Duration,
    pub(crate) completed: CompletedDomainChecks,
    pub(crate) include_suppressed: bool,
    pub(crate) unproven_member_query_sql: u32,
    pub(crate) sql_verdicts: Vec<no_mistakes::codebase::postgres::SqlObligationVerdict>,
    pub(crate) include_sql_verdicts: bool,
}

pub(crate) struct CheckResults {
    pub(crate) incomplete: Vec<no_mistakes::codebase::check_outcome::CheckFailure>,
    pub(crate) react: Vec<react_traits::Violation>,
    pub(crate) queues: Vec<CheckFinding>,
    pub(crate) rules: Vec<RuleFinding>,
    pub(crate) integration: Vec<IntegrationFinding>,
    pub(crate) runner_config_deadlines:
        Option<Vec<no_mistakes::integration_tests::RunnerConfigDeadlineEvidence>>,
    pub(crate) codebase: Vec<UniqueExportFinding>,
    pub(crate) warnings: Vec<String>,
    pub(crate) analysis_warning_count: usize,
    pub(crate) analysis_fallbacks: Vec<no_mistakes::config::v2::AnalysisFallback>,
    pub(crate) advisories: Vec<RuleFinding>,
    pub(crate) sql_verdicts: Vec<no_mistakes::codebase::postgres::SqlObligationVerdict>,
    pub(crate) include_sql_verdicts: bool,
    pub(crate) suppressed: Vec<no_mistakes::codebase::rules::SuppressedFinding>,
    pub(crate) include_suppressed: bool,
    pub(crate) timings: Vec<(&'static str, Duration)>,
    pub(crate) unproven_member_query_sql: u32,
    pub(crate) accepted_bounds: Vec<no_mistakes::codebase::check_facts::AcceptedStatementBound>,
}

pub(crate) fn finalize_domain_checks(input: FinalizeInput<'_>) -> Result<CheckResults> {
    let FinalizeInput {
        root,
        facts,
        runner_config_deadlines,
        config,
        filesystem_files,
        disable_directives,
        sources,
        filesystem_rules_enabled,
        react_warning,
        discover_duration,
        facts_duration,
        mut completed,
        include_suppressed,
        unproven_member_query_sql,
        sql_verdicts,
        include_sql_verdicts,
    } = input;
    let pre_suppressed = suppression::take_pre_suppressed(&mut completed);
    let CompletedDomainChecks {
        mut react,
        mut queues,
        mut rules,
        mut integration,
        mut codebase,
        mut filesystem_rules,
    } = completed;
    let react_suppression_targets = react.react_suppression_targets.as_slice();
    let rule_suppression_sources = rules.suppression_sources.as_slice();
    let audit_directives = config.audits_disable_directives();
    let retain_suppression = include_suppressed || audit_directives;
    let mut advisories = advisories::collect(
        filesystem_rules_enabled,
        retain_suppression,
        root,
        config,
        filesystem_files,
        sources,
    )?;
    advisories::merge_sql(&mut advisories, root, sources, facts, &sql_verdicts);
    advisories.extend(std::mem::take(&mut rules.advisories));
    advisories.extend(std::mem::take(&mut filesystem_rules.advisories));
    let (warnings, analysis_warning_count) = empty::collect_warnings(
        root,
        facts,
        config,
        [
            react_warning,
            react.warning,
            queues.warning,
            rules.warning,
            integration.warning,
            codebase.warning,
            filesystem_rules.warning,
        ],
    );
    let accounting = suppression::apply_if_requested(
        retain_suppression,
        suppression::Inputs {
            pre_suppressed,
            root,
            sources,
            react: &mut react.findings,
            react_suppression_targets: retain_suppression.then_some(react_suppression_targets),
            queues: &mut queues.findings,
            rules: &mut rules.findings,
            rule_suppression_sources,
            filesystem: &mut filesystem_rules.findings,
            integration: &mut integration.findings,
            codebase: &mut codebase.findings,
            advisories: &mut advisories,
        },
    );
    let directive_findings = disable_directives.findings_with_postgres(
        config,
        filesystem_files,
        sources,
        facts,
        &accounting.consumed,
    )?;
    let suppressed = accounting.into_public(include_suppressed);
    if config.report_unused_disable_directives == DisableDirectivePolicy::Report {
        rules.findings.extend(directive_findings);
    } else {
        advisories.extend(directive_findings);
    }
    // The public `rules` list has historically contained both codebase and
    // filesystem findings in that domain order. Preserve it while retaining
    // the distinct adapter identity in optional suppression accounting.
    rules.findings.extend(filesystem_rules.findings);
    advisories::demote(
        config,
        &mut queues.findings,
        &mut rules.findings,
        &mut codebase.findings,
        &mut advisories,
    );
    let codebase_findings = codebase.findings.into_iter().map(|p| p.finding).collect();
    Ok(CheckResults {
        incomplete: Vec::new(),
        timings: empty::timings([
            discover_duration,
            facts_duration,
            react.duration,
            queues.duration,
            rules.duration,
            integration.duration,
            codebase.duration,
            filesystem_rules.duration,
        ]),
        react: react.findings,
        queues: queues.findings,
        rules: rules.findings,
        integration: integration.findings,
        runner_config_deadlines,
        codebase: codebase_findings,
        warnings,
        analysis_warning_count,
        analysis_fallbacks: config.analysis_fallbacks.clone(),
        advisories,
        sql_verdicts,
        include_sql_verdicts,
        suppressed,
        include_suppressed,
        unproven_member_query_sql,
        accepted_bounds: facts.accepted_statement_bounds(),
    })
}
