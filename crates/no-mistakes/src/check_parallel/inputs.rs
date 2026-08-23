use crate::check_tasks::CheckTask;
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::unique_exports::PreparedUniqueExportFinding;
use no_mistakes::integration_tests::IntegrationFinding;
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;
use std::path::{Path, PathBuf};

pub(crate) type DomainResults = (
    anyhow::Result<CheckTask<Vec<react_traits::Violation>>>,
    anyhow::Result<CheckTask<Vec<CheckFinding>>>,
    anyhow::Result<CheckTask<Vec<RuleFinding>>>,
    anyhow::Result<CheckTask<Vec<IntegrationFinding>>>,
    anyhow::Result<CheckTask<Vec<PreparedUniqueExportFinding>>>,
    anyhow::Result<CheckTask<Vec<RuleFinding>>>,
);

pub(crate) struct DomainCheckInputs<'a> {
    pub(crate) session: std::sync::Arc<no_mistakes::codebase::analysis_session::AnalysisSession>,
    pub(crate) root: &'a Path,
    pub(crate) config_path: &'a Option<PathBuf>,
    pub(crate) tsconfig_path: &'a Option<PathBuf>,
    pub(crate) react_enabled: bool,
    pub(crate) queues_enabled: bool,
    pub(crate) integration_enabled: bool,
    pub(crate) unique_exports_enabled: bool,
    pub(crate) filesystem_rules_enabled: bool,
    pub(crate) discovered_files: &'a [PathBuf],
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
    pub(crate) workflow_documents:
        Option<&'a no_mistakes::codebase::ci_workflows::ParsedWorkflowSet>,
    pub(crate) tsconfig_gate_project_inputs:
        Option<&'a no_mistakes::codebase::rules::tsconfig_gate_coverage::ProjectSourceInputs>,
    pub(crate) defer_suppression: bool,
}
