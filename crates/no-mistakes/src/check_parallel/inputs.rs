use crate::check_tasks::CheckTask;
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::unique_exports::PreparedUniqueExportFinding;
use no_mistakes::integration_tests::IntegrationFinding;
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;
use std::path::{Path, PathBuf};

pub type DomainResults = (
    anyhow::Result<CheckTask<Vec<react_traits::Violation>>>,
    anyhow::Result<CheckTask<Vec<CheckFinding>>>,
    anyhow::Result<CheckTask<Vec<RuleFinding>>>,
    anyhow::Result<CheckTask<Vec<IntegrationFinding>>>,
    anyhow::Result<CheckTask<Vec<PreparedUniqueExportFinding>>>,
    anyhow::Result<CheckTask<Vec<RuleFinding>>>,
);

pub struct DomainCheckInputs<'a> {
    pub session: std::sync::Arc<no_mistakes::codebase::analysis_session::AnalysisSession>,
    pub root: &'a Path,
    pub config_path: &'a Option<PathBuf>,
    pub tsconfig_path: &'a Option<PathBuf>,
    pub react_enabled: bool,
    pub queues_enabled: bool,
    pub integration_enabled: bool,
    pub unique_exports_enabled: bool,
    pub filesystem_rules_enabled: bool,
    pub discovered_files: &'a [PathBuf],
    pub facts: &'a CheckFactMap,
    pub prepared_playwright: Option<&'a no_mistakes::playwright::rules::PreparedPlaywrightRules>,
    pub prepared_react: &'a no_mistakes::react_traits::PreparedReactCheck,
    pub prepared_graph: Option<&'a no_mistakes::codebase::dependencies::graph::PreparedGraphConfig>,
    pub dependency_graph:
        Option<std::sync::Arc<no_mistakes::codebase::dependencies::graph::DepGraph>>,
    pub prepared_tsconfig: &'a no_mistakes::codebase::ts_resolver::TsConfig,
    pub prepared_tsconfig_catalog:
        &'a std::sync::Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    pub visible_paths: &'a no_mistakes::codebase::ts_source::VisiblePathSnapshot,
    pub sources: std::sync::Arc<no_mistakes::codebase::ts_source::SourceStore>,
    pub inferred_roots: &'a no_mistakes::codebase::config::InferredRoots,
    pub config: &'a no_mistakes::config::v2::NoMistakesConfig,
    pub codebase_config: &'a no_mistakes::codebase::config::Config,
    pub vitest_projects: Option<&'a no_mistakes::codebase::rules::PreparedVitestProjectCatalog>,
    pub playwright_projects:
        Option<&'a no_mistakes::codebase::rules::PreparedPlaywrightProjectCatalog>,
    pub workflow_documents:
        Option<&'a no_mistakes::codebase::ci_workflows::PreparedWorkflowTopologyInputs>,
    pub tsconfig_gate_project_inputs:
        Option<&'a no_mistakes::codebase::rules::tsconfig_gate_coverage::ProjectSourceInputs>,
    pub defer_suppression: bool,
}
