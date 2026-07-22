use anyhow::Result;
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::codebase::rules::{self, RuleFinding};
use no_mistakes::codebase::unique_exports::{self, UniqueExportFinding};
use no_mistakes::config::v2::NoMistakesConfig;
use no_mistakes::integration_tests::{self, IntegrationFinding};
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;
use std::time::Duration;

mod filesystem;
#[cfg(test)]
mod tests;

pub(crate) use filesystem::{filesystem_rules_configured, run_filesystem_rules_check};

pub(crate) struct CheckTask<T> {
    pub(crate) findings: T,
    pub(crate) warning: Option<String>,
    pub(crate) duration: Duration,
}

pub(crate) fn run_react_check(
    root: &std::path::Path,
    enabled: bool,
    facts: &CheckFactMap,
    prepared: &react_traits::PreparedReactCheck,
) -> Result<CheckTask<Vec<react_traits::Violation>>> {
    let ((findings, warning), duration) = no_mistakes::diagnostics::measure_if_enabled(
        "analysis.react",
        no_mistakes::diagnostics::TimingKind::Parallel,
        || {
            if enabled {
                match react_traits::run_check_with_prepared_facts(root, &[], facts, prepared) {
                    Ok(findings) => (findings, None),
                    Err(err) => (
                        Vec::new(),
                        Some(format!("warning: react check skipped: {err:#}")),
                    ),
                }
            } else {
                (Vec::new(), None)
            }
        },
    );
    Ok(CheckTask {
        findings,
        warning,
        duration,
    })
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
        warning: None,
        duration,
    })
}

pub(crate) fn run_rules_check(
    inputs: rules::PreparedRulesCheck<'_>,
    dependency_graph: Option<&no_mistakes::codebase::dependencies::graph::DepGraph>,
) -> Result<CheckTask<Vec<RuleFinding>>> {
    let ((findings, warning), duration) = no_mistakes::diagnostics::measure_if_enabled(
        "analysis.rules",
        no_mistakes::diagnostics::TimingKind::Parallel,
        || match rules::run_check_with_config_facts_playwright_and_graph(inputs, dependency_graph) {
            Ok(findings) => (findings, None),
            Err(err) => (
                Vec::new(),
                Some(format!("warning: rules check skipped: {err:#}")),
            ),
        },
    );
    Ok(CheckTask {
        findings,
        warning,
        duration,
    })
}

pub(crate) fn run_integration_check(
    session: &no_mistakes::codebase::analysis_session::AnalysisSession,
    root: &std::path::Path,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
    tsconfig_catalog: &std::sync::Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    visible_paths: &no_mistakes::codebase::ts_source::VisiblePathSnapshot,
) -> Result<CheckTask<Vec<IntegrationFinding>>> {
    let (findings, duration) = no_mistakes::diagnostics::measure_if_enabled(
        "analysis.integration",
        no_mistakes::diagnostics::TimingKind::Parallel,
        || {
            integration_tests::check_with_prepared_facts_catalog_and_session(
                root,
                config,
                facts,
                std::sync::Arc::clone(tsconfig_catalog),
                visible_paths,
                session,
            )
        },
    );
    let findings = findings?;
    Ok(CheckTask {
        findings,
        warning: None,
        duration,
    })
}

pub(crate) fn run_codebase_check_with_catalog(
    session: &no_mistakes::codebase::analysis_session::AnalysisSession,
    root: &std::path::Path,
    config: &no_mistakes::codebase::config::Config,
    prepared_tsconfig_catalog: &std::sync::Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    enabled: bool,
    facts: &CheckFactMap,
    inferred_roots: &no_mistakes::codebase::config::InferredRoots,
) -> Result<CheckTask<Vec<UniqueExportFinding>>> {
    let (findings, duration) = no_mistakes::diagnostics::measure_if_enabled(
        "analysis.codebase",
        no_mistakes::diagnostics::TimingKind::Parallel,
        || -> Result<_> {
            Ok(if enabled {
                unique_exports::analyze_project_with_prepared_facts_catalog_and_inferred_and_session(
                    root,
                    config,
                    prepared_tsconfig_catalog,
                    facts,
                    inferred_roots,
                    session,
                )?
            } else {
                Vec::new()
            })
        },
    );
    let findings = findings?;
    Ok(CheckTask {
        findings,
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

pub(crate) fn forbidden_dependencies_configured(config: &NoMistakesConfig) -> bool {
    rule_configured(config, rules::FORBIDDEN_DEPENDENCIES)
}

pub(crate) fn unique_exports_configured(config: &NoMistakesConfig) -> bool {
    rule_configured(config, unique_exports::RULE_ID)
}

pub(crate) fn rule_configured(config: &NoMistakesConfig, rule_id: &str) -> bool {
    config.rule_configured(rule_id)
}
