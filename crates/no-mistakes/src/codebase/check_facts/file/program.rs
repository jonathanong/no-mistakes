use super::super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use super::plan::{ts_extract_context, ts_extract_plan};
use super::should_store_source;
use crate::codebase::ts_source::facts;
use std::path::Path;
use std::sync::Arc;

pub(crate) fn collect_file_facts_from_program(
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    owned_source: Option<Arc<str>>,
) -> CheckFileFacts {
    let stored_source =
        owned_source.or_else(|| should_store_source(plan).then(|| Arc::<str>::from(source)));
    let ts = facts::collect_file_facts_from_program(
        path,
        ts_extract_plan(plan, path, playwright),
        &ts_extract_context(root, plan),
        source,
        program,
        None,
        stored_source.clone(),
    );
    let react = (plan.react || plan.graph.react).then(|| {
        Arc::new(crate::react_traits::analyze::file::FileAnalysis {
            components: Arc::clone(&ts.react_components),
        })
    });
    let react_usages = plan.react_usages.then(|| {
        crate::react_traits::pipeline::usages::collect_usage_file_facts(
            path,
            source,
            program,
            plan.graph_context.visible_files.as_deref(),
        )
    });
    let integration = plan
        .integration
        .then(|| crate::integration_tests::analysis::analyze_program(path, program, source));
    let integration_runner_config = plan
        .integration_runner_configs
        .as_ref()
        .and_then(|plan| plan.parse_program(path, program, source));
    let fused = super::program_walk::collect_fused_check_program(path, source, program, plan);
    let server_route_client_boundary = plan.server_route_client_boundary.then(|| {
        crate::codebase::rules::server_route_client_boundary::extract_program(path, source, program)
    });
    let playwright_fetch = playwright
        .filter(|plan| plan.contains_source(path))
        .map(|plan| {
            let mut import_cache = std::collections::HashMap::new();
            crate::fetch::file_facts::ParsedFileFacts::from_program(
                path,
                root,
                source,
                program,
                &mut import_cache,
                plan.source_file_set(),
            )
        });
    let playwright_source =
        super::playwright_source::collect(root, path, source, program, playwright);
    let playwright_static_exports = playwright_fetch
        .as_ref()
        .map(|_| crate::playwright::selectors::collect_static_export_values(program));
    let playwright =
        super::super::file_playwright::collect_playwright_facts(path, program, source, playwright);
    let embedded_sql = prepared_embedded_sql(path, source, program, plan);
    let symbols = ts.symbols.clone();
    CheckFileFacts {
        ts: ts.into(),
        source: stored_source,
        symbols,
        legacy_symbols: None,
        react,
        react_usages,
        integration,
        integration_runner_config,
        dynamic_imports: fused.dynamic_imports,
        nextjs_caching: fused.nextjs_caching,
        storybook: fused.storybook,
        embedded_sql,
        server_route_client_boundary,
        playwright,
        playwright_fetch,
        playwright_app_selectors: playwright_source.selectors,
        playwright_app_text_targets: playwright_source.text_targets,
        playwright_static_exports,
        parse_error: None,
        legacy_symbol_parse_error: None,
        parsed: true,
    }
}

pub(super) fn prepared_embedded_sql(
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    plan: &CheckFactPlan,
) -> Vec<(
    crate::codebase::postgres::EmbeddedSqlOptions,
    crate::codebase::postgres::EmbeddedSqlFileFacts,
)> {
    let mut profiles = plan.embedded_sql_options.clone();
    if plan.embedded_sql && profiles.is_empty() {
        profiles.push(crate::codebase::postgres::EmbeddedSqlOptions::default());
    }
    profiles.sort();
    profiles.dedup();
    profiles
        .into_iter()
        .map(|options| {
            let facts = crate::codebase::postgres::extract_embedded_sql_from_program(
                path, program, source, &options,
            );
            (options, facts)
        })
        .collect()
}
