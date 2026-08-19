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
            components: Arc::new(ts.react_components.clone()),
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
    let dynamic_imports = plan.dynamic_imports.then(|| {
        crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::extract_program(
            source, program,
        )
    });
    let nextjs_caching = plan
        .nextjs_caching
        .then(|| crate::codebase::rules::nextjs_no_caching::extract_program(path, source, program));
    let storybook = plan
        .storybook
        .then(|| crate::codebase::storybook::extract_program(source, program));
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
        dynamic_imports,
        nextjs_caching,
        storybook,
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
