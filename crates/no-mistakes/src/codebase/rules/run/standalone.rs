use super::{
    any_codebase_rule_enabled, forbidden_dependencies, rule_enabled, PreparedRulesCheck,
    FORBIDDEN_DEPENDENCIES, NEXTJS_NO_API_ROUTES, NEXTJS_NO_CACHING, REQUIRE_STORYBOOK_STORIES,
    SERVER_ROUTE_CLIENT_BOUNDARY, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
};
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

pub(super) fn run_check(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<super::RuleFinding>> {
    let snapshot = Arc::new(crate::codebase::ts_source::VisiblePathSnapshot::new(root));
    let visible_paths = snapshot.paths_for(root);
    let config = crate::config::v2::load_v2_config_from_visible(root, config_path, &visible_paths)?;
    if !any_codebase_rule_enabled(&config) {
        return Ok(Vec::new());
    }

    let inferred_roots = crate::codebase::config::InferredRoots::from_visible(root, &visible_paths);
    let prepared_tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        tsconfig_path,
        root,
        &visible_paths,
    )?;
    let prepared_playwright = crate::playwright::rules::prepare_from_snapshot(
        root,
        config_path,
        &config,
        Arc::clone(&snapshot),
        Arc::new(prepared_tsconfig.clone()),
    )?;
    let graph_plan = rule_enabled(&config, FORBIDDEN_DEPENDENCIES)
        .then(|| forbidden_dependencies::graph_plan(&config))
        .flatten();
    let codebase_config =
        crate::codebase::config::config_from_loaded_v2(root, config_path, &config);
    let prepared_graph = graph_plan
        .map(|plan| {
            crate::codebase::dependencies::graph::prepare_graph_config(
                root,
                plan,
                &codebase_config,
                &config,
                snapshot.as_ref(),
            )
        })
        .transpose()?;

    let mut fact_plan = standalone_fact_plan(&config);
    if let (Some(plan), Some(prepared)) = (graph_plan, prepared_graph.as_ref()) {
        let (graph_facts, graph_context) =
            crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
                root, plan, prepared,
            );
        fact_plan.graph = graph_facts;
        fact_plan.graph_context = graph_context;
    }
    let files = crate::codebase::ts_source::discover_files_from_visible(
        root,
        &config.filesystem.skip_directories,
        &visible_paths,
    );
    let graph_files = if graph_plan.is_some() {
        crate::codebase::ts_source::discover_files_from_visible(root, &[], &visible_paths)
    } else if rule_enabled(&config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS) {
        files.clone()
    } else {
        Vec::new()
    };
    let playwright_fact_plan = prepared_playwright
        .as_ref()
        .map(crate::playwright::rules::PreparedPlaywrightRules::fact_plan);
    let sources = snapshot.source_store_for(root);
    let prepared_tsconfig_catalog = super::prepared_tsconfig_catalog(
        root,
        tsconfig_path,
        &prepared_tsconfig,
        &visible_paths,
        &sources,
        Some(&config),
    );
    let shared = collect_check_facts_with_graph_files_playwright_and_sources(
        root,
        files,
        graph_files,
        fact_plan,
        playwright_fact_plan,
        Arc::clone(&sources),
    );

    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    super::run_check_with_config_and_facts_and_playwright(PreparedRulesCheck {
        session,
        root,
        config_path,
        tsconfig_path,
        shared: &shared,
        prepared_playwright: prepared_playwright.as_ref(),
        config: &config,
        prepared_graph: prepared_graph.as_ref(),
        prepared_tsconfig: &prepared_tsconfig,
        prepared_tsconfig_catalog: &prepared_tsconfig_catalog,
        inferred_roots: Some(&inferred_roots),
        sources: Some(&sources),
    })
}

fn standalone_fact_plan(config: &crate::config::v2::NoMistakesConfig) -> CheckFactPlan {
    let dynamic_imports = rule_enabled(config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS);
    let boundary = rule_enabled(config, SERVER_ROUTE_CLIENT_BOUNDARY);
    let nextjs_api_routes = rule_enabled(config, NEXTJS_NO_API_ROUTES);
    let nextjs_caching = rule_enabled(config, NEXTJS_NO_CACHING);
    let storybook = rule_enabled(config, REQUIRE_STORYBOOK_STORIES);
    CheckFactPlan {
        imports: dynamic_imports,
        symbols: storybook,
        react: storybook,
        dynamic_imports: dynamic_imports || storybook,
        nextjs_caching,
        storybook,
        server_route_client_boundary: boundary,
        raw_source: nextjs_api_routes,
        source: dynamic_imports || nextjs_caching || storybook,
        ..CheckFactPlan::default()
    }
}
