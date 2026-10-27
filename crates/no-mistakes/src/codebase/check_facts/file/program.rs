use super::super::file_playwright::collect_playwright_facts;
use super::super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use super::plan::ts_extract_plan;
use super::should_store_source;
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
    let embedded_sql = prepared_embedded_sql(root, path, source, program, plan);
    let mut head_candidates = heads::candidates(plan, &embedded_sql);
    let helpers = embedded_sql
        .iter()
        .any(|(profile, _)| !profile.sql_literal_helpers.is_empty());
    if helpers {
        head_candidates.extend(
            embedded_sql
                .iter()
                .filter(|(profile, _)| !profile.sql_literal_helpers.is_empty())
                .flat_map(|(_, file)| file.call_spans.iter().copied()),
        );
    }
    let loop_heads = (plan.query_annotation
        && !plan.query_annotation_heads
        && heads::has_annotation_executors(plan, &embedded_sql))
    .then(|| {
        crate::codebase::postgres::query_annotation::heads::prepare_loop_heads(
            program,
            source,
            !head_candidates.is_empty(),
            helpers,
        )
    })
    .flatten();
    let mut ts_plan = ts_extract_plan(root, plan, path, playwright);
    ts_plan.module_bindings |= !head_candidates.is_empty() || loop_heads.is_some();
    let stored_source = owned_source.or_else(|| {
        (should_store_source(plan, path)
            || super::sql_identifier::policy_selected(root, path, plan))
        .then(|| Arc::<str>::from(source))
    });
    let super::prepared_program::PreparedProgramFacts { ts, semantic } =
        super::prepared_program::collect(
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

    let react = ((plan.react
        && crate::codebase::check_facts::family_scope::applies(&plan.react_paths, path))
        || plan.graph.react)
        .then(|| {
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
    let integration = (plan.integration
        && crate::codebase::check_facts::family_scope::applies(&plan.integration_paths, path))
    .then(|| crate::integration_tests::analysis::analyze_program(path, program, source));
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
    let playwright_source = super::playwright_source::collect(
        root,
        path,
        source,
        program,
        playwright,
        &ts.imported_bindings,
    );
    let playwright_static_exports = playwright_fetch
        .as_ref()
        .map(|_| crate::playwright::selectors::collect_static_export_values(program));
    let playwright = collect_playwright_facts(path, program, source, playwright, &semantic);
    let query_annotation = if plan.query_annotation {
        let options = plan.query_annotation_options.clone().unwrap_or_else(|| {
            embedded_sql
                .iter()
                .map(|(options, _)| options.clone())
                .collect()
        });
        options
            .iter()
            .map(|options| {
                (
                    options.clone(),
                    crate::codebase::postgres::query_annotation::collect(program, source, options),
                )
            })
            .collect()
    } else {
        Vec::new()
    };
    let query_annotation_heads = (plan.query_annotation_heads
        || !head_candidates.is_empty()
        || loop_heads.is_some())
    .then(|| {
        Arc::new(
            crate::codebase::postgres::query_annotation::heads::collect_prepared(
                program,
                &ts,
                embedded_sql
                    .iter()
                    .flat_map(|(_, file)| file.call_spans.iter().copied()),
                &head_candidates,
                plan.query_annotation_heads,
                helpers,
                loop_heads,
            ),
        )
    });
    let sql_identifier_syntax =
        super::sql_identifier::collect(root, path, source, program, &ts, plan);
    let symbols = ts.symbols.clone();
    CheckFileFacts {
        lint_suppressions: plan.collect_lint_suppressions(path, program, source),
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
        embedded_sql,
        query_annotation,
        query_annotation_heads,
        sql_identifier_syntax,
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

#[path = "program_embedded.rs"]
mod embedded;
pub(super) use embedded::prepared_embedded_sql;
#[path = "program_heads.rs"]
mod heads;
