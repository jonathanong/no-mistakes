use super::super::CheckFileFacts;
use super::plan::ts_extract_plan;
use super::should_store_source;
use super::CheckFactVariant;
use crate::codebase::playwright_hooks;
use crate::codebase::ts_source::facts::react_analysis::{self, component::FileAnalysis};
use std::path::Path;
use std::sync::Arc;
pub(crate) fn collect_file_facts_from_program_with_discovery(
    variant: &CheckFactVariant<'_>,
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    owned_source: Option<Arc<str>>,
) -> CheckFileFacts {
    let sql = crate::codebase::postgres_analysis::FileInput {
        root: variant.root,
        path,
        source,
        program,
        plan: variant.plan,
    };
    (crate::codebase::postgres_analysis::hooks().collect_program)(variant, sql, owned_source)
}

pub(crate) fn collect_with_postgres(
    variant: &CheckFactVariant<'_>,
    input: crate::codebase::postgres_analysis::FileInput<'_, '_>,
    owned_source: Option<Arc<str>>,
    sql_bindings: bool,
    finish_sql: impl FnOnce(
        &crate::codebase::ts_source::facts::TsFileFacts,
    ) -> crate::codebase::postgres_analysis::FilePayload,
) -> CheckFileFacts {
    use crate::codebase::rules::{csp_allowed_origins, nested_controls};
    let crate::codebase::postgres_analysis::FileInput {
        root,
        path,
        source,
        program,
        plan,
    } = input;
    let playwright = variant.playwright;
    let mut ts_plan = ts_extract_plan(root, plan, path, playwright);
    ts_plan.module_bindings |= sql_bindings;
    let stored_source = owned_source.or_else(|| {
        (should_store_source(plan, path)
            || (crate::codebase::postgres_analysis::hooks().retain_source)(root, path, plan))
        .then(|| Arc::<str>::from(source))
    });
    let prepared = super::prepared_program::collect(
        root,
        path,
        super::prepared_program::Demand {
            request: plan,
            ts: ts_plan,
        },
        playwright,
        source,
        program,
        stored_source.clone(),
    );
    let ts = prepared.ts;
    let react = ((plan.react
        && crate::codebase::check_facts::family_scope::applies(&plan.react_paths, path))
        || plan.graph.react)
        .then(|| {
            Arc::new(FileAnalysis {
                components: Arc::clone(&ts.react_components),
            })
        });
    let react_usages = plan.react_usages.then(|| {
        (react_analysis::hooks().extract_usages)(
            path,
            source,
            program,
            plan.graph_context.visible_files.as_deref(),
        )
    });
    let integration = (plan.integration
        && crate::codebase::check_facts::family_scope::applies(&plan.integration_paths, path))
    .then(|| {
        crate::integration_tests::analysis::analyze_program_with_test_discovery(
            path,
            program,
            source,
            variant.collect_tests,
        )
    });
    let integration_runner_config = plan
        .integration_runner_configs
        .as_ref()
        .and_then(|plan| plan.parse_program(path, program, source));
    let fused = super::program_walk::collect_fused_check_program(path, source, program, plan);
    let server_route_client_boundary = plan.server_route_client_boundary.then(|| {
        if crate::codebase::check_facts::family_scope::applies(&plan.boundary_paths, path) {
            crate::codebase::rules::server_route_client_boundary::extract_program(
                path, source, program,
            )
        } else {
            Default::default()
        }
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
    let (playwright_source, playwright_static_exports, playwright) = playwright
        .map(|plan| {
            let hooks = playwright_hooks::hooks();
            let bindings = &ts.imported_bindings;
            let source_facts = (hooks.source_facts)(root, path, source, program, plan, bindings);
            let static_exports = playwright_fetch
                .as_ref()
                .map(|_| (hooks.static_exports)(program));
            let test_facts = (hooks.test_facts)(path, program, source, plan, &prepared.semantic);
            (source_facts, static_exports, test_facts)
        })
        .unwrap_or_default();
    let postgres_file = finish_sql(&ts);
    let symbols = ts.symbols.clone();
    CheckFileFacts {
        nested_controls: nested_controls::collect(path, source, program, plan),
        fact_extraction_panicked: false,
        test_reliability: prepared.test_reliability,
        fixture_initializers: prepared.fixture_initializers,
        lint_suppressions: plan.collect_lint_suppressions(path, program, source),
        vitest_default_excludes: prepared.vitest_default_excludes,
        csp_allowed_origins: csp_allowed_origins::collect_for_plan(path, source, program, plan),
        ts: ts.into(),
        production_imports: None,
        source: stored_source,
        symbols,
        legacy_symbols: None,
        react,
        react_usages,
        integration,
        integration_runner_config,
        dynamic_imports: fused.dynamic_imports,
        nextjs_caching: fused.nextjs_caching,
        nextjs_caching_low: fused.nextjs_caching_low,
        storybook: fused.storybook,
        postgres_file,
        sql_source_prerequisites: Default::default(),
        server_route_client_boundary,
        playwright,
        playwright_fetch,
        playwright_app_selectors: playwright_source.selectors,
        playwright_app_text_targets: playwright_source.text_targets,
        playwright_static_exports,
        integration_route_occurrences: playwright_source.integration_routes,
        parse_error: None,
        legacy_symbol_parse_error: None,
        parsed: true,
    }
}
