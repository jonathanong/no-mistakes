use super::{
    forbidden_dependencies, nextjs_no_api_routes, nextjs_no_caching, require_storybook_stories,
    rule_enabled, server_route_client_boundary, sort_findings, suppress_rule_findings,
    suppress_rule_findings_with_sources, test_no_unmocked_dynamic_imports, RuleFinding,
    FORBIDDEN_DEPENDENCIES, NEXTJS_NO_API_ROUTES, NEXTJS_NO_CACHING, REQUIRE_STORYBOOK_STORIES,
    SERVER_ROUTE_CLIENT_BOUNDARY, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
};
use anyhow::Result;
use std::path::Path;

mod prepared;
mod standalone;

pub(crate) use prepared::canonical_graph_plan;
#[doc(hidden)]
pub use prepared::run_check_with_config_facts_playwright_and_graph;
pub use prepared::{run_check_with_config_and_facts_and_playwright, PreparedRulesCheck};

pub fn run_check(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    standalone::run_check(root, config_path, tsconfig_path)
}

pub fn run_check_with_facts(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    run_check_with_facts_and_playwright(root, config_path, tsconfig_path, shared, None)
}

pub fn run_check_with_facts_and_playwright(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
    shared: &crate::codebase::check_facts::CheckFactMap,
    prepared_playwright: Option<&crate::playwright::rules::PreparedPlaywrightRules>,
) -> Result<Vec<RuleFinding>> {
    let config = crate::config::v2::load_v2_config(root, config_path)?;
    let prepared_tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        tsconfig_path,
        root,
        shared.files(),
    )?;
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    run_check_with_config_and_facts_and_playwright(PreparedRulesCheck {
        session,
        root,
        config_path,
        tsconfig_path,
        shared,
        prepared_playwright,
        config: &config,
        prepared_graph: None,
        prepared_tsconfig: &prepared_tsconfig,
        inferred_roots: None,
        sources: None,
    })
}

fn any_codebase_rule_enabled(config: &crate::config::v2::NoMistakesConfig) -> bool {
    rule_enabled(config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS)
        || rule_enabled(config, SERVER_ROUTE_CLIENT_BOUNDARY)
        || rule_enabled(config, NEXTJS_NO_API_ROUTES)
        || rule_enabled(config, NEXTJS_NO_CACHING)
        || rule_enabled(config, REQUIRE_STORYBOOK_STORIES)
        || crate::playwright::rules::configured(config)
        || rule_enabled(config, FORBIDDEN_DEPENDENCIES)
}
