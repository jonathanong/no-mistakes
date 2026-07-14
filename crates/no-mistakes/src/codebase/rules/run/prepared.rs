use super::{
    any_codebase_rule_enabled, forbidden_dependencies, nextjs_no_api_routes, nextjs_no_caching,
    require_storybook_stories, rule_enabled, server_route_client_boundary, sort_findings,
    suppress_rule_findings, test_no_unmocked_dynamic_imports, RuleFinding, FORBIDDEN_DEPENDENCIES,
    NEXTJS_NO_API_ROUTES, NEXTJS_NO_CACHING, REQUIRE_STORYBOOK_STORIES,
    SERVER_ROUTE_CLIENT_BOUNDARY, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use anyhow::Result;
use std::path::Path;

mod execution;

/// Preloaded inputs for the aggregate rules check.
///
/// This keeps the aggregate `check` path from reloading configuration while
/// leaving the standalone rule-check entry points unchanged.
#[doc(hidden)]
pub struct PreparedRulesCheck<'a> {
    pub root: &'a Path,
    pub config_path: Option<&'a Path>,
    pub tsconfig_path: Option<&'a Path>,
    pub shared: &'a crate::codebase::check_facts::CheckFactMap,
    pub prepared_playwright: Option<&'a crate::playwright::rules::PreparedPlaywrightRules>,
    pub config: &'a crate::config::v2::NoMistakesConfig,
    pub prepared_graph: Option<&'a crate::codebase::dependencies::graph::PreparedGraphConfig>,
    pub prepared_tsconfig: &'a crate::codebase::ts_resolver::TsConfig,
    pub inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
}

/// Shared-config entry point used by the aggregate `check` command.
#[doc(hidden)]
pub(crate) fn canonical_graph_plan(
    config: &crate::config::v2::NoMistakesConfig,
) -> Option<GraphBuildPlan> {
    let mut plan = GraphBuildPlan::default();
    let mut needed = false;
    if rule_enabled(config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS) {
        plan.include(GraphBuildPlan::imports_and_workspace());
        needed = true;
    }
    if let Some(forbidden_plan) = forbidden_dependencies::graph_plan(config) {
        plan.include(forbidden_plan);
        needed = true;
    }
    needed.then_some(plan)
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
    execution::run(inputs, dependency_graph)
}
