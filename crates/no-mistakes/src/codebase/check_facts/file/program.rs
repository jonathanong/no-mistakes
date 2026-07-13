use super::super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use super::should_store_source;
use crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source;
use crate::codebase::ts_source::facts::{self, TsFileFacts};
use crate::codebase::ts_symbols::extract_symbols_from_program;
use std::path::Path;

pub(crate) fn collect_file_facts_from_program(
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
) -> CheckFileFacts {
    let needs_import_facts = plan.imports
        || plan.graph.imports
        || plan.graph.function_calls
        || playwright.is_some_and(|plan| plan.file(path).is_some());
    let import_facts = if needs_import_facts {
        extract_import_facts_from_program_with_source(program, source)
    } else {
        Default::default()
    };
    let symbols = if plan.symbols || plan.graph.symbols {
        Some(extract_symbols_from_program(program, source))
    } else {
        None
    };
    let react = if plan.react || plan.graph.react {
        Some(match plan.graph_context.visible_files.as_deref() {
            Some(visible) => crate::react_traits::analyze::file::analyze_program_from_visible(
                path, root, source, program, visible,
            ),
            None => {
                crate::react_traits::analyze::file::analyze_program(path, root, source, program)
            }
        })
    } else {
        None
    };
    let react_usages = plan.react_usages.then(|| {
        crate::react_traits::pipeline::usages::collect_usage_file_facts(
            path,
            source,
            program,
            plan.graph_context.visible_files.as_deref(),
        )
    });
    let queue = if plan.queue || plan.graph.queue_project {
        Some(crate::queue::extract::extract_program_with_factories(
            path,
            source,
            program,
            &plan.queue_factory_names,
        ))
    } else {
        None
    };
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
    let domain = if plan.graph.has_domain_facts() {
        facts::domain::collect_domain_facts(program, path, source, plan.graph, &plan.graph_context)
    } else {
        facts::domain::DomainFacts::default()
    };
    let queue_project = queue.or(domain.queue_project);
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
    let ts = TsFileFacts {
        source: should_store_source(plan).then(|| source.to_string()),
        parse_error: None,
        imports: import_facts.imports,
        function_calls: import_facts.function_calls,
        symbol_references: import_facts.symbol_references,
        exported_functions: import_facts.exported_functions,
        unknown_callers: import_facts.unknown_callers,
        has_unknown_top_level_call: import_facts.has_unknown_top_level_call,
        symbols: symbols.clone(),
        route_refs: domain.route_refs,
        route_helpers: domain.route_helpers,
        route_helper_imports: domain.route_helper_imports,
        route_helper_refs: domain.route_helper_refs,
        backend_routes: domain.backend_routes,
        queue_usage: domain.queue_usage,
        queue_create_line: domain.queue_create_line,
        queue_name: domain.queue_name,
        queue_project,
        http_calls: domain.http_calls,
        process_spawns: domain.process_spawns,
        server_routes: domain.server_routes,
        effect_calls: domain.effect_calls,
        rsc_environment: domain.rsc_environment,
        react_components: react
            .as_ref()
            .map(|analysis| analysis.components.clone())
            .unwrap_or_default(),
    };
    CheckFileFacts {
        ts,
        source: should_store_source(plan).then(|| source.to_string()),
        symbols,
        react,
        react_usages,
        integration,
        integration_runner_config,
        dynamic_imports,
        nextjs_caching,
        storybook,
        playwright,
        playwright_fetch,
        playwright_app_selectors: playwright_source.selectors,
        playwright_app_text_targets: playwright_source.text_targets,
        playwright_static_exports,
        parse_error: None,
        parsed: true,
    }
}
