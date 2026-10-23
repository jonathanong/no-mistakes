use crate::check_parallel::DomainResults;
use crate::check_tasks::CheckTask;
use anyhow::Result;
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::unique_exports::{PreparedUniqueExportFinding, UniqueExportFinding};
use no_mistakes::integration_tests::IntegrationFinding;
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;
use std::time::Duration;

mod advisories;
mod empty;
pub(crate) use empty::empty_results;
mod render;
pub(crate) use render::json_value;
pub(crate) mod suppression;
#[cfg(test)]
mod suppression_tests;

pub(crate) struct FinalizeInput<'a> {
    pub(crate) root: &'a std::path::Path,
    pub(crate) runner_config_deadlines:
        Option<Vec<no_mistakes::integration_tests::RunnerConfigDeadlineEvidence>>,
    pub(crate) config: &'a no_mistakes::config::v2::NoMistakesConfig,
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
    /// Deduped SQL-looking member `.query` calls with no executor proof.
    /// Omitted from JSON when zero. Not part of `warnings`.
    pub(crate) unproven_member_query_sql: u32,
}

pub(crate) struct CompletedDomainChecks {
    pub(crate) react: CheckTask<Vec<react_traits::Violation>>,
    pub(crate) queues: CheckTask<Vec<CheckFinding>>,
    pub(crate) rules: CheckTask<Vec<RuleFinding>>,
    pub(crate) integration: CheckTask<Vec<IntegrationFinding>>,
    pub(crate) codebase: CheckTask<Vec<PreparedUniqueExportFinding>>,
    pub(crate) filesystem_rules: CheckTask<Vec<RuleFinding>>,
}

pub(crate) fn complete_domain_checks(results: DomainResults) -> Result<CompletedDomainChecks> {
    let (react, queues, rules, integration, codebase, filesystem_rules) = results;
    Ok(CompletedDomainChecks {
        react: react?,
        queues: queues?,
        rules: rules?,
        integration: integration?,
        codebase: codebase?,
        filesystem_rules: filesystem_rules?,
    })
}

pub(crate) fn finalize_domain_checks(input: FinalizeInput<'_>) -> Result<CheckResults> {
    let FinalizeInput {
        root,
        runner_config_deadlines,
        config,
        filesystem_files,
        sources,
        filesystem_rules_enabled,
        react_warning,
        discover_duration,
        facts_duration,
        completed,
        include_suppressed,
        unproven_member_query_sql,
        sql_verdicts,
        include_sql_verdicts,
    } = input;
    let mut react = completed.react;
    let react_suppression_targets = react.react_suppression_targets.as_slice();
    let mut queues = completed.queues;
    let mut rules = completed.rules;
    let rule_suppression_sources = rules.suppression_sources.as_slice();
    let mut integration = completed.integration;
    let mut codebase = completed.codebase;
    let mut filesystem_rules = completed.filesystem_rules;
    let advisories = advisories::collect(
        filesystem_rules_enabled,
        include_suppressed,
        root,
        config,
        filesystem_files,
        sources,
    );
    let mut advisories = advisories?;
    advisories::merge_sql(&mut advisories, root, sources, &sql_verdicts);
    advisories.extend(std::mem::take(&mut rules.advisories));
    advisories.extend(std::mem::take(&mut filesystem_rules.advisories));
    let mut warnings: Vec<String> = [
        react_warning,
        react.warning,
        queues.warning,
        rules.warning,
        integration.warning,
        codebase.warning,
        filesystem_rules.warning,
    ]
    .into_iter()
    .flatten()
    .collect();
    let analysis_warning_count = config.analysis_warnings.len();
    warnings.extend(config.analysis_warnings.iter().cloned());
    let suppressed = suppression::apply_if_requested(
        include_suppressed,
        suppression::Inputs {
            root,
            sources,
            react: &mut react.findings,
            react_suppression_targets: include_suppressed.then_some(react_suppression_targets),
            queues: &mut queues.findings,
            rules: &mut rules.findings,
            rule_suppression_sources,
            filesystem: &mut filesystem_rules.findings,
            integration: &mut integration.findings,
            codebase: &mut codebase.findings,
            advisories: &mut advisories,
        },
    );
    // The public `rules` list has historically contained both codebase and
    // filesystem findings in that domain order. Preserve it while retaining
    // the distinct adapter identity in optional suppression accounting.
    rules.findings.extend(filesystem_rules.findings);
    Ok(CheckResults {
        timings: vec![
            ("discover", discover_duration),
            ("parse_extract", facts_duration),
            ("react", react.duration),
            ("queues", queues.duration),
            ("rules", rules.duration),
            ("integration", integration.duration),
            ("codebase", codebase.duration),
            ("filesystem_rules", filesystem_rules.duration),
        ],
        react: react.findings,
        queues: queues.findings,
        rules: rules.findings,
        integration: integration.findings,
        runner_config_deadlines,
        codebase: codebase
            .findings
            .into_iter()
            .map(|prepared| prepared.finding)
            .collect(),
        warnings,
        analysis_warning_count,
        analysis_fallbacks: config.analysis_fallbacks.clone(),
        advisories,
        sql_verdicts,
        include_sql_verdicts,
        suppressed: if include_suppressed {
            suppressed
        } else {
            Vec::new()
        },
        include_suppressed,
        unproven_member_query_sql,
    })
}
