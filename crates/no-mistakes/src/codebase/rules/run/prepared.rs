use super::{
    any_codebase_rule_enabled, forbidden_dependencies, nextjs_no_api_routes, nextjs_no_caching,
    require_storybook_stories, required_entrypoint_reachability, rule_enabled,
    server_route_client_boundary, suppress_rule_findings_with_sources,
    test_no_unmocked_dynamic_imports, PreparedRuleFindings, RuleFinding, FORBIDDEN_DEPENDENCIES,
    NEXTJS_NO_API_ROUTES, NEXTJS_NO_CACHING, REQUIRED_ENTRYPOINT_REACHABILITY,
    REQUIRE_STORYBOOK_STORIES, SERVER_ROUTE_CLIENT_BOUNDARY, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use anyhow::Result;
use std::path::Path;

mod execution;
#[cfg(test)]
mod tests;

/// Preloaded inputs for the aggregate rules check.
///
/// This keeps the aggregate `check` path from reloading configuration while
/// leaving the standalone rule-check entry points unchanged.
#[doc(hidden)]
pub struct PreparedRulesCheck<'a> {
    pub session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    pub root: &'a Path,
    pub config_path: Option<&'a Path>,
    pub tsconfig_path: Option<&'a Path>,
    pub shared: &'a crate::codebase::check_facts::CheckFactMap,
    pub prepared_playwright: Option<&'a crate::playwright::rules::PreparedPlaywrightRules>,
    pub config: &'a crate::config::v2::NoMistakesConfig,
    pub prepared_graph: Option<&'a crate::codebase::dependencies::graph::PreparedGraphConfig>,
    pub prepared_tsconfig: &'a crate::codebase::ts_resolver::TsConfig,
    pub prepared_tsconfig_catalog: &'a crate::codebase::ts_resolver::TsConfigCatalog,
    pub inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
    pub sources: Option<&'a crate::codebase::ts_source::SourceStore>,
}

/// Shared-config entry point used by the aggregate `check` command.
#[doc(hidden)]
pub fn canonical_graph_plan(
    config: &crate::config::v2::NoMistakesConfig,
) -> Result<Option<GraphBuildPlan>> {
    let mut plan = GraphBuildPlan::default();
    let mut needed = false;
    if rule_enabled(config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS) {
        plan.include(GraphBuildPlan::imports_and_workspace());
        needed = true;
    }
    if let Some(reachability_plan) = required_entrypoint_reachability::graph_plan(config) {
        plan.include(reachability_plan);
        needed = true;
    }
    if let Some(forbidden_plan) = forbidden_dependencies::graph_plan(config)? {
        plan.include(forbidden_plan);
        needed = true;
    }
    Ok(needed.then_some(plan))
}

/// Whether configured graph-backed rules require files outside the filesystem check scope.
#[doc(hidden)]
pub fn canonical_graph_requires_full_file_universe(
    config: &crate::config::v2::NoMistakesConfig,
) -> bool {
    required_entrypoint_reachability::graph_plan(config).is_some()
        || config.rule_configured(FORBIDDEN_DEPENDENCIES)
}

pub fn run_check_with_config_and_facts_and_playwright(
    inputs: PreparedRulesCheck<'_>,
) -> Result<Vec<RuleFinding>> {
    run_check_with_config_facts_playwright_and_graph(inputs, None)
}

pub fn run_check_with_config_facts_playwright_and_graph(
    inputs: PreparedRulesCheck<'_>,
    dependency_graph: Option<&DepGraph>,
) -> Result<Vec<RuleFinding>> {
    Ok(execution::run(inputs, dependency_graph, None, false)?.findings)
}

#[doc(hidden)]
pub fn run_check_with_config_facts_playwright_and_graph_with_suppression(
    inputs: PreparedRulesCheck<'_>,
    dependency_graph: Option<&DepGraph>,
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<PreparedRuleFindings> {
    execution::run(inputs, dependency_graph, Some(sources), defer_suppression)
}
