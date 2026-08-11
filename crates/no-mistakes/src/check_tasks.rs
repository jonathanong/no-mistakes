use anyhow::Result;
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::codebase::rules::{self, RuleFinding};
use no_mistakes::codebase::unique_exports::{self, PreparedUniqueExportFinding};
use no_mistakes::config::v2::NoMistakesConfig;
use no_mistakes::integration_tests::{self, IntegrationFinding};
use no_mistakes::queue::CheckFinding;
use std::time::Duration;

mod filesystem;
mod react;
#[cfg(test)]
mod tests;

pub(crate) use filesystem::{filesystem_rules_configured, run_filesystem_rules_check_with_facts};
pub(crate) use react::run_react_check;

pub(crate) struct CheckTask<T> {
    pub(crate) findings: T,
    pub(crate) react_suppression_targets:
        Vec<Vec<no_mistakes::react_traits::ReactSuppressionTarget>>,
    pub(crate) suppression_sources: Vec<Option<String>>,
    pub(crate) warning: Option<String>,
    pub(crate) duration: Duration,
}

pub(crate) fn run_queue_check(
    root: &std::path::Path,
    prepared_tsconfig_catalog: &std::sync::Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    enabled: bool,
    facts: &CheckFactMap,
    session: &no_mistakes::codebase::analysis_session::AnalysisSession,
) -> Result<CheckTask<Vec<CheckFinding>>> {
    let (findings, duration) = no_mistakes::diagnostics::measure_if_enabled(
        "analysis.queues",
        no_mistakes::diagnostics::TimingKind::Parallel,
        || -> Result<_> {
            Ok(if enabled {
                no_mistakes::queue::analyze_project_with_prepared_facts_and_catalog_and_session(
                    root,
                    prepared_tsconfig_catalog,
                    &[],
                    facts,
                    session,
                )?
                .check
            } else {
                Vec::new()
            })
        },
    );
    let findings = findings?;
    Ok(CheckTask {
        findings,
        react_suppression_targets: Vec::new(),
        suppression_sources: Vec::new(),
        warning: None,
        duration,
    })
}

pub(crate) fn run_rules_check(
    inputs: rules::PreparedRulesCheck<'_>,
    dependency_graph: Option<&no_mistakes::codebase::dependencies::graph::DepGraph>,
    sources: &no_mistakes::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<CheckTask<Vec<RuleFinding>>> {
    let (((findings, suppression_sources), warning), duration) =
        no_mistakes::diagnostics::measure_if_enabled(
            "analysis.rules",
            no_mistakes::diagnostics::TimingKind::Parallel,
            || match rules::run_check_with_config_facts_playwright_and_graph_with_suppression(
                inputs,
                dependency_graph,
                sources,
                defer_suppression,
            ) {
                Ok(findings) => ((findings.findings, findings.suppression_sources), None),
                Err(err) => (
                    (Vec::new(), Vec::new()),
                    Some(format!("warning: rules check skipped: {err:#}")),
                ),
            },
        );
    Ok(CheckTask {
        findings,
        react_suppression_targets: Vec::new(),
        suppression_sources,
        warning,
        duration,
    })
}

pub(crate) fn run_integration_check(
    session: &no_mistakes::codebase::analysis_session::AnalysisSession,
    root: &std::path::Path,
    enabled: bool,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
    tsconfig_catalog: &std::sync::Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    visible_paths: &no_mistakes::codebase::ts_source::VisiblePathSnapshot,
) -> Result<CheckTask<Vec<IntegrationFinding>>> {
    let (findings, duration) = no_mistakes::diagnostics::measure_if_enabled(
        "analysis.integration",
        no_mistakes::diagnostics::TimingKind::Parallel,
        || {
            if enabled {
                integration_tests::check_with_prepared_facts_catalog_and_session(
                    root,
                    config,
                    facts,
                    std::sync::Arc::clone(tsconfig_catalog),
                    visible_paths,
                    session,
                )
            } else {
                Ok(Vec::new())
            }
        },
    );
    let findings = findings?;
    Ok(CheckTask {
        findings,
        react_suppression_targets: Vec::new(),
        suppression_sources: Vec::new(),
        warning: None,
        duration,
    })
}

pub(crate) struct CodebaseCheckInputs<'a> {
    pub(crate) session: &'a no_mistakes::codebase::analysis_session::AnalysisSession,
    pub(crate) root: &'a std::path::Path,
    pub(crate) config: &'a no_mistakes::codebase::config::Config,
    pub(crate) prepared_tsconfig_catalog:
        &'a std::sync::Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    pub(crate) enabled: bool,
    pub(crate) facts: &'a CheckFactMap,
    pub(crate) inferred_roots: &'a no_mistakes::codebase::config::InferredRoots,
    pub(crate) defer_suppression: bool,
}

pub(crate) fn run_codebase_check_with_catalog(
    inputs: CodebaseCheckInputs<'_>,
) -> Result<CheckTask<Vec<PreparedUniqueExportFinding>>> {
    let CodebaseCheckInputs {
        session,
        root,
        config,
        prepared_tsconfig_catalog,
        enabled,
        facts,
        inferred_roots,
        defer_suppression,
    } = inputs;
    let (findings, duration) = no_mistakes::diagnostics::measure_if_enabled(
        "analysis.codebase",
        no_mistakes::diagnostics::TimingKind::Parallel,
        || -> Result<_> {
            Ok(if enabled {
                unique_exports::analyze_project_with_prepared_facts_catalog_and_inferred_and_session_for_check(
                    root,
                    config,
                    prepared_tsconfig_catalog,
                    facts,
                    inferred_roots,
                    session,
                    defer_suppression,
                )?
            } else {
                Vec::new()
            })
        },
    );
    let findings = findings?;
    Ok(CheckTask {
        findings,
        react_suppression_targets: Vec::new(),
        suppression_sources: Vec::new(),
        warning: None,
        duration,
    })
}

pub(crate) fn queues_configured(config: &NoMistakesConfig) -> bool {
    config
        .projects
        .values()
        .any(|project| !project.queues.enqueues.is_empty() || !project.queues.workers.is_empty())
}

pub(crate) fn unique_exports_configured(config: &NoMistakesConfig) -> bool {
    rule_configured(config, unique_exports::RULE_ID)
}

pub(crate) fn rule_configured(config: &NoMistakesConfig, rule_id: &str) -> bool {
    config.rule_configured(rule_id)
}
