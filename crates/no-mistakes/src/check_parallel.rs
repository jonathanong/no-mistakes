use crate::check_tasks::{
    run_codebase_check_with_catalog, run_filesystem_rules_check, run_integration_check,
    run_queue_check, run_react_check, run_rules_check, CheckTask,
};
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::unique_exports::UniqueExportFinding;
use no_mistakes::integration_tests::IntegrationFinding;
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;
use std::path::{Path, PathBuf};

pub(crate) type DomainResults = (
    anyhow::Result<CheckTask<Vec<react_traits::Violation>>>,
    anyhow::Result<CheckTask<Vec<CheckFinding>>>,
    anyhow::Result<CheckTask<Vec<RuleFinding>>>,
    anyhow::Result<CheckTask<Vec<IntegrationFinding>>>,
    anyhow::Result<CheckTask<Vec<UniqueExportFinding>>>,
    anyhow::Result<CheckTask<Vec<RuleFinding>>>,
);

pub(crate) struct DomainCheckInputs<'a> {
    pub(crate) session: std::sync::Arc<no_mistakes::codebase::analysis_session::AnalysisSession>,
    pub(crate) root: &'a Path,
    pub(crate) config_path: &'a Option<PathBuf>,
    pub(crate) tsconfig_path: &'a Option<PathBuf>,
    pub(crate) react_enabled: bool,
    pub(crate) queues_enabled: bool,
    pub(crate) unique_exports_enabled: bool,
    pub(crate) filesystem_rules_enabled: bool,
    pub(crate) discovered_files: Vec<PathBuf>,
    pub(crate) facts: &'a CheckFactMap,
    pub(crate) prepared_playwright:
        Option<&'a no_mistakes::playwright::rules::PreparedPlaywrightRules>,
    pub(crate) prepared_react: &'a no_mistakes::react_traits::PreparedReactCheck,
    pub(crate) prepared_graph:
        Option<&'a no_mistakes::codebase::dependencies::graph::PreparedGraphConfig>,
    pub(crate) dependency_graph:
        Option<std::sync::Arc<no_mistakes::codebase::dependencies::graph::DepGraph>>,
    pub(crate) prepared_tsconfig: &'a no_mistakes::codebase::ts_resolver::TsConfig,
    pub(crate) prepared_tsconfig_catalog:
        &'a std::sync::Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    pub(crate) visible_paths: &'a no_mistakes::codebase::ts_source::VisiblePathSnapshot,
    pub(crate) sources: std::sync::Arc<no_mistakes::codebase::ts_source::SourceStore>,
    pub(crate) inferred_roots: &'a no_mistakes::codebase::config::InferredRoots,
    pub(crate) config: &'a no_mistakes::config::v2::NoMistakesConfig,
    pub(crate) codebase_config: &'a no_mistakes::codebase::config::Config,
    pub(crate) vitest_projects:
        Option<&'a no_mistakes::codebase::rules::PreparedVitestProjectCatalog>,
}

pub(crate) fn run_domain_checks(inputs: DomainCheckInputs<'_>) -> DomainResults {
    let observer = no_mistakes::diagnostics::current();
    let session = inputs.session;
    let root = inputs.root;
    let config_path = inputs.config_path;
    let tsconfig_path = inputs.tsconfig_path;
    let react_enabled = inputs.react_enabled;
    let queues_enabled = inputs.queues_enabled;
    let unique_exports_enabled = inputs.unique_exports_enabled;
    let filesystem_rules_enabled = inputs.filesystem_rules_enabled;
    let discovered_files = inputs.discovered_files;
    let facts = inputs.facts;
    let prepared_playwright = inputs.prepared_playwright;
    let prepared_react = inputs.prepared_react;
    let prepared_graph = inputs.prepared_graph;
    let dependency_graph = inputs.dependency_graph;
    let prepared_tsconfig = inputs.prepared_tsconfig;
    let prepared_tsconfig_catalog = inputs.prepared_tsconfig_catalog;
    let visible_paths = inputs.visible_paths;
    let sources = inputs.sources;
    let rule_sources = std::sync::Arc::clone(&sources);
    let inferred_roots = inputs.inferred_roots;
    let config = inputs.config;
    let codebase_config = inputs.codebase_config;
    let vitest_projects = inputs.vitest_projects;

    let ((react, queues), (rules, (integration, (codebase, filesystem_rules)))) = rayon::join(
        || {
            rayon::join(
                || {
                    no_mistakes::diagnostics::with_observer(observer.clone(), || {
                        run_react_check(root, react_enabled, facts, prepared_react)
                    })
                },
                || {
                    no_mistakes::diagnostics::with_observer(observer.clone(), || {
                        run_queue_check(
                            root,
                            prepared_tsconfig_catalog,
                            queues_enabled,
                            facts,
                            &session,
                        )
                    })
                },
            )
        },
        || {
            rayon::join(
                || {
                    no_mistakes::diagnostics::with_observer(observer.clone(), || {
                        run_rules_check(
                            no_mistakes::codebase::rules::PreparedRulesCheck {
                                session: session.clone(),
                                root,
                                config_path: config_path.as_deref(),
                                tsconfig_path: tsconfig_path.as_deref(),
                                shared: facts,
                                prepared_playwright,
                                config,
                                prepared_graph,
                                prepared_tsconfig,
                                prepared_tsconfig_catalog,
                                inferred_roots: Some(inferred_roots),
                                sources: Some(&rule_sources),
                            },
                            dependency_graph.as_deref(),
                        )
                    })
                },
                || {
                    rayon::join(
                        || {
                            no_mistakes::diagnostics::with_observer(observer.clone(), || {
                                run_integration_check(
                                    &session,
                                    root,
                                    config,
                                    facts,
                                    prepared_tsconfig_catalog,
                                    visible_paths,
                                )
                            })
                        },
                        || {
                            rayon::join(
                                || {
                                    no_mistakes::diagnostics::with_observer(
                                        observer.clone(),
                                        || {
                                            run_codebase_check_with_catalog(
                                                &session,
                                                root,
                                                codebase_config,
                                                prepared_tsconfig_catalog,
                                                unique_exports_enabled,
                                                facts,
                                                inferred_roots,
                                            )
                                        },
                                    )
                                },
                                || {
                                    no_mistakes::diagnostics::with_observer(
                                        observer.clone(),
                                        || {
                                            run_filesystem_rules_check(
                                                root,
                                                config,
                                                filesystem_rules_enabled,
                                                &discovered_files,
                                                visible_paths,
                                                sources,
                                                vitest_projects,
                                            )
                                        },
                                    )
                                },
                            )
                        },
                    )
                },
            )
        },
    );
    (
        react,
        queues,
        rules,
        integration,
        codebase,
        filesystem_rules,
    )
}
