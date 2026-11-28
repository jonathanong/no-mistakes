use anyhow::Result;
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::codebase::rules::{self, RuleFinding};
use no_mistakes::codebase::unique_exports::{self, PreparedUniqueExportFinding};
use no_mistakes::config::v2::NoMistakesConfig;
use no_mistakes::integration_tests::{self, IntegrationFinding};

mod configured;
pub use configured::{queues_configured, rule_configured, unique_exports_configured};
mod filesystem;
mod queues;
mod react;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;

pub use filesystem::{filesystem_rules_configured, run_filesystem_rules_check_with_facts};
pub use queues::run_queue_check;
pub use react::run_react_check_with_session;

mod types;
pub use types::CheckTask;

pub fn run_rules_check(
    inputs: rules::PreparedRulesCheck<'_>,
    dependency_graph: Option<&no_mistakes::codebase::dependencies::graph::DepGraph>,
    sources: &no_mistakes::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<CheckTask<Vec<RuleFinding>>> {
    let (((findings, suppression_sources, advisories), warning), duration) =
        no_mistakes::diagnostics::measure_if_enabled(
            "analysis.rules",
            no_mistakes::diagnostics::TimingKind::Parallel,
            || match rules::run_check_with_config_facts_playwright_and_graph_with_suppression(
                inputs,
                dependency_graph,
                sources,
                defer_suppression,
            ) {
                Ok(findings) => (
                    (
                        findings.findings,
                        findings.suppression_sources,
                        findings.advisories,
                    ),
                    None,
                ),
                Err(err) => (
                    (Vec::new(), Vec::new(), Vec::new()),
                    Some(format!("warning: rules check skipped: {err:#}")),
                ),
            },
        );
    Ok(CheckTask {
        findings,
        pre_suppressed: Vec::new(),
        react_suppression_targets: Vec::new(),
        suppression_sources,
        advisories,
        analysis_warnings: Vec::new(),
        warning,
        duration,
    })
}

pub fn run_integration_check(
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
        pre_suppressed: Vec::new(),
        react_suppression_targets: Vec::new(),
        suppression_sources: Vec::new(),
        advisories: Vec::new(),
        analysis_warnings: Vec::new(),
        warning: None,
        duration,
    })
}

pub struct CodebaseCheckInputs<'a> {
    pub session: &'a no_mistakes::codebase::analysis_session::AnalysisSession,
    pub root: &'a std::path::Path,
    pub config: &'a no_mistakes::codebase::config::Config,
    pub prepared_tsconfig_catalog:
        &'a std::sync::Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    pub enabled: bool,
    pub facts: &'a CheckFactMap,
    pub inferred_roots: &'a no_mistakes::codebase::config::InferredRoots,
    pub defer_suppression: bool,
}

pub fn run_codebase_check_with_catalog(
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
        pre_suppressed: Vec::new(),
        react_suppression_targets: Vec::new(),
        suppression_sources: Vec::new(),
        advisories: Vec::new(),
        analysis_warnings: Vec::new(),
        warning: None,
        duration,
    })
}
