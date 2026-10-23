mod graph_files;
use super::{
    any_codebase_rule_enabled, rule_enabled, try_canonical_graph_plan, PreparedRulesCheck,
    NEXTJS_NO_API_ROUTES, NEXTJS_NO_CACHING, REQUIRE_STORYBOOK_STORIES,
    SERVER_ROUTE_CLIENT_BOUNDARY, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
};
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use anyhow::Result;
use graph_files::standalone_graph_files;
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

    let files = crate::codebase::ts_source::discover_files_from_visible(
        root,
        &config.filesystem.skip_directories,
        &visible_paths,
    );
    let sources = snapshot.source_store_for(root);
    let config =
        super::require_storybook_stories::prepare_low_config(root, &config, &files, &sources)?
            .unwrap_or(config);

    let inferred_roots = crate::codebase::config::InferredRoots::from_visible(root, &visible_paths);
    let prepared_tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        tsconfig_path,
        root,
        &visible_paths,
    );
    let prepared_tsconfig = prepared_tsconfig?;
    let prepared_playwright = crate::playwright::rules::prepare_from_snapshot(
        root,
        config_path,
        &config,
        Arc::clone(&snapshot),
        Arc::new(prepared_tsconfig.clone()),
    );
    let prepared_playwright = prepared_playwright?;
    let graph_plan = try_canonical_graph_plan(&config)?;
    let codebase_config =
        crate::codebase::config::config_from_loaded_v2(root, config_path, &config);
    let prepared_graph = graph_plan
        .map(|plan| {
            let test_filter = crate::codebase::test_filter::TestFileFilter::new(root, &config);
            crate::codebase::dependencies::graph::prepare_graph_config_with_test_filter(
                root,
                plan,
                &codebase_config,
                &config,
                snapshot.as_ref(),
                test_filter,
            )
        })
        .transpose()?;

    let mut fact_plan = standalone_fact_plan(&config)?;
    crate::codebase::postgres::configure_prepared_postgres_plan(root, &config, &mut fact_plan)?;
    crate::codebase::postgres::assign_prepared_embedded_sql_options(&config, &mut fact_plan)?;
    fact_plan.embedded_sql = !fact_plan.embedded_sql_options.is_empty();
    fact_plan.postgres_schema_catalog_paths =
        crate::codebase::postgres::configured_schema_catalog_paths(
            &config,
            crate::codebase::postgres::SCHEMA_CATALOG_RULE_IDS,
        )?;
    if let (Some(plan), Some(prepared)) = (graph_plan, prepared_graph.as_ref()) {
        let (graph_facts, graph_context) =
            crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
                root, plan, prepared,
            );
        fact_plan.graph = graph_facts;
        fact_plan.graph_context = graph_context;
    }
    fact_plan.graph.per_item_calls |= config.rule_configured(super::QUERY_REACHED_PER_ITEM);
    fact_plan.graph.call_arguments |=
        crate::codebase::rules::forbidden_calls::argument_facts_required(&config);
    fact_plan.production_import_paths =
        super::super::production_dependency_declarations::required_import_fact_files(
            root,
            &config,
            &visible_paths,
        )?;
    fact_plan.graph.imports |= !fact_plan.production_import_paths.is_empty();

    let graph_files = standalone_graph_files(root, &config, graph_plan, &visible_paths, &files);
    let playwright_fact_plan = prepared_playwright
        .as_ref()
        .map(crate::playwright::rules::PreparedPlaywrightRules::fact_plan);

    crate::codebase::check_facts::configure_family_path_scopes(
        root,
        &files,
        &config,
        sources.as_ref(),
        &mut fact_plan,
    )?;
    let prepared_tsconfig_catalog = Arc::new(super::prepared_tsconfig_catalog(
        root,
        tsconfig_path,
        &prepared_tsconfig,
        &visible_paths,
        &sources,
        Some(&config),
    ));
    fact_plan.query_annotation_catalog = Some(Arc::clone(&prepared_tsconfig_catalog));
    if rule_enabled(&config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS) {
        fact_plan.integration_runner_configs = Some(Arc::new(
            crate::integration_tests::prepare_runner_configs_with_catalog(
                root,
                &config,
                &visible_paths,
                Arc::clone(&prepared_tsconfig_catalog),
                Arc::clone(&sources),
            ),
        ));
    }
    let (prepared_vitest_projects, prepared_playwright_projects) =
        crate::codebase::rules::forbidden_calls::prepare_root_catalogs(
            root,
            &config,
            snapshot.as_ref(),
            &prepared_tsconfig_catalog,
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
        prepared_vitest_projects: prepared_vitest_projects.as_ref(),
        prepared_playwright_projects: prepared_playwright_projects.as_ref(),
        inferred_roots: Some(&inferred_roots),
        sources: Some(&sources),
    })
}

#[cfg(test)]
mod tests;

fn standalone_fact_plan(config: &crate::config::v2::NoMistakesConfig) -> Result<CheckFactPlan> {
    let dynamic_imports = rule_enabled(config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS);
    let boundary = rule_enabled(config, SERVER_ROUTE_CLIENT_BOUNDARY);
    let nextjs_api_routes = rule_enabled(config, NEXTJS_NO_API_ROUTES);
    let nextjs_caching = rule_enabled(config, NEXTJS_NO_CACHING);
    let storybook = rule_enabled(config, REQUIRE_STORYBOOK_STORIES);
    let mut plan = CheckFactPlan {
        imports: dynamic_imports,
        symbols: storybook,
        react: storybook,
        dynamic_imports,
        nextjs_caching,
        storybook,
        server_route_client_boundary: boundary,
        raw_source: nextjs_api_routes,
        source: dynamic_imports || nextjs_caching || storybook,
        ..CheckFactPlan::default()
    };
    super::nextjs_no_caching::configure_effort_plan(config, &mut plan)?;
    super::require_storybook_stories::configure_effort_plan(config, &mut plan)?;
    Ok(plan)
}
