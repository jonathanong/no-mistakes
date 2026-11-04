use super::*;

pub(in crate::codebase::rules::run::prepared::execution) struct GraphRuleRequest<'a> {
    pub(in crate::codebase::rules::run::prepared::execution) root: &'a Path,
    pub(in crate::codebase::rules::run::prepared::execution) config:
        &'a crate::config::v2::NoMistakesConfig,
    pub(in crate::codebase::rules::run::prepared::execution) config_path: Option<&'a Path>,
    pub(in crate::codebase::rules::run::prepared::execution) shared:
        &'a crate::codebase::check_facts::CheckFactMap,
    pub(in crate::codebase::rules::run::prepared::execution) prepared_graph:
        Option<&'a crate::codebase::dependencies::graph::PreparedGraphConfig>,
    pub(in crate::codebase::rules::run::prepared::execution) dependency_graph: Option<&'a DepGraph>,
    pub(in crate::codebase::rules::run::prepared::execution) inferred_roots:
        Option<&'a crate::codebase::config::InferredRoots>,
    pub(in crate::codebase::rules::run::prepared::execution) sources:
        Option<&'a crate::codebase::ts_source::SourceStore>,
    pub(in crate::codebase::rules::run::prepared::execution) tsconfig_catalog:
        Option<&'a crate::codebase::ts_resolver::TsConfigCatalog>,
    pub(in crate::codebase::rules::run::prepared::execution) prepared_vitest_projects:
        Option<&'a crate::codebase::rules::PreparedVitestProjectCatalog>,
    pub(in crate::codebase::rules::run::prepared::execution) prepared_playwright_projects:
        Option<&'a crate::codebase::rules::PreparedPlaywrightProjectCatalog>,
}
