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
    // Prepared facts are the caller's immutable request universe. Rebuilding a
    // snapshot here could discover a different set of package tsconfigs than
    // the graph facts were collected from.
    let visible = shared.files();
    let sources = crate::codebase::rules::source_store_for_files(visible);
    let prepared_tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
            tsconfig_path,
            root,
            visible,
            &sources,
        )?;
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let prepared_tsconfig_catalog = prepared_tsconfig_catalog(
        root,
        tsconfig_path,
        &prepared_tsconfig,
        visible,
        &sources,
        Some(&config),
    );
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
        prepared_tsconfig_catalog: &prepared_tsconfig_catalog,
        inferred_roots: None,
        sources: Some(&sources),
    })
}

pub(crate) fn prepared_tsconfig_catalog(
    root: &Path,
    tsconfig_path: Option<&Path>,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    visible_paths: &[std::path::PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    config: Option<&crate::config::v2::NoMistakesConfig>,
) -> crate::codebase::ts_resolver::TsConfigCatalog {
    if let Some(path) = tsconfig_path {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        crate::codebase::ts_resolver::TsConfigCatalog::forced(
            root,
            tsconfig.clone(),
            Some(crate::codebase::ts_resolver::normalize_path(&path)),
        )
    } else {
        let mut candidate_roots = vec![root.to_path_buf()];
        if let Some(config) = config {
            candidate_roots.extend(require_storybook_stories::configured_project_roots(
                root, config,
            ));
        }
        crate::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources(
            root,
            &candidate_roots,
            visible_paths,
            sources,
        )
    }
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
