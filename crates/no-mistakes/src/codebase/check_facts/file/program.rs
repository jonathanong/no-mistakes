use super::super::file_playwright::collect_playwright_facts;
use super::super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
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
    let stored_source =
        owned_source.or_else(|| should_store_source(plan).then(|| Arc::<str>::from(source)));
    let super::prepared_program::PreparedProgramFacts { ts, semantic } =
        super::prepared_program::collect(
            root,
            path,
            plan,
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
    let embedded_sql = prepared_embedded_sql(root, path, source, program, plan);
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
    let query_annotation_heads = plan.query_annotation_heads.then(|| {
        Arc::new(crate::codebase::postgres::query_annotation::heads::collect(
            program,
            &ts,
            embedded_sql
                .iter()
                .flat_map(|(_, file)| file.call_spans.iter().copied()),
        ))
    });
    let sql_identifier_syntax =
        super::sql_identifier::collect(root, path, source, program, &ts, plan);
    let symbols = ts.symbols.clone();
    CheckFileFacts {
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

pub(super) fn prepared_embedded_sql(
    root: &Path,
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
    let rel = crate::codebase::ts_source::relative_slash_path(root, path);
    profiles
        .into_iter()
        .map(|options| {
            let mut facts = crate::codebase::postgres::extract_embedded_sql_from_program(
                path, program, source, &options,
            );
            assign_unproven_rules(
                &mut facts,
                plan.embedded_sql_rule_ids.get(&options).map(Vec::as_slice),
                &rel,
                source,
            );
            (options, facts)
        })
        .collect()
}

fn assign_unproven_rules(
    facts: &mut crate::codebase::postgres::EmbeddedSqlFileFacts,
    scans: Option<&[crate::codebase::postgres::EmbeddedSqlRuleScan]>,
    rel: &str,
    source: &str,
) {
    let Some(scans) = scans else {
        crate::codebase::postgres::retain_unproven_spans(facts, source);
        return;
    };
    let mut ids = Vec::new();
    for scan in scans {
        if scan.paths.includes(rel) && !ids.contains(&scan.rule_id) {
            ids.push(scan.rule_id.clone());
        }
    }
    if ids.is_empty() {
        facts.unproven_member_query_spans.clear();
        facts.record_unproven_spans = false;
        return;
    }
    facts.postgres_rule_ids = ids;
    crate::codebase::postgres::retain_unproven_spans(facts, source);
}
