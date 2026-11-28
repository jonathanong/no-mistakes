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
pub use partial::retain_domain_checks;
mod empty;
pub use empty::{empty_results, CompletedDomainChecks};
mod render;
pub use render::json_value;
pub mod suppression;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod suppression_tests;
mod warnings;

pub struct FinalizeInput<'a> {
    pub root: &'a std::path::Path,
    pub facts: &'a no_mistakes::codebase::check_facts::CheckFactMap,
    pub runner_config_deadlines:
        Option<Vec<no_mistakes::integration_tests::RunnerConfigDeadlineEvidence>>,
    pub config: &'a no_mistakes::config::v2::NoMistakesConfig,
    pub disable_directives:
        &'a no_mistakes::codebase::rules::disable_directive_audit::PreparedDisableDirectives,
    pub filesystem_files: &'a [std::path::PathBuf],
    pub sources: &'a no_mistakes::codebase::ts_source::SourceStore,
    pub filesystem_rules_enabled: bool,
    pub react_warning: Option<String>,
    pub discover_duration: Duration,
    pub facts_duration: Duration,
    pub completed: CompletedDomainChecks,
    pub include_suppressed: bool,
    pub unproven_member_query_sql: u32,
    pub sql_verdicts: Vec<no_mistakes::codebase::postgres::SqlObligationVerdict>,
    pub include_sql_verdicts: bool,
}

pub struct CheckResults {
    pub incomplete: Vec<no_mistakes::codebase::check_outcome::CheckFailure>,
    pub react: Vec<react_traits::Violation>,
    pub queues: Vec<CheckFinding>,
    pub rules: Vec<RuleFinding>,
    pub integration: Vec<IntegrationFinding>,
    pub runner_config_deadlines:
        Option<Vec<no_mistakes::integration_tests::RunnerConfigDeadlineEvidence>>,
    pub codebase: Vec<UniqueExportFinding>,
    pub warnings: Vec<String>,
    pub analysis_warning_count: usize,
    pub analysis_fallbacks: Vec<no_mistakes::config::v2::AnalysisFallback>,
    pub advisories: Vec<RuleFinding>,
    pub sql_verdicts: Vec<no_mistakes::codebase::postgres::SqlObligationVerdict>,
    pub include_sql_verdicts: bool,
    pub suppressed: Vec<no_mistakes::codebase::rules::SuppressedFinding>,
    pub include_suppressed: bool,
    pub timings: Vec<(&'static str, Duration)>,
    pub unproven_member_query_sql: u32,
    pub accepted_bounds: Vec<no_mistakes::codebase::check_facts::AcceptedStatementBound>,
}

pub fn finalize_domain_checks(input: FinalizeInput<'_>) -> Result<CheckResults> {
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
    let warnings::CollectedWarnings {
        warnings,
        analysis_warning_count,
    } = warnings::collect(root, facts, config, react_warning, &mut completed);
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
