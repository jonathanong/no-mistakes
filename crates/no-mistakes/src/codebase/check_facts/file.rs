use super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use crate::codebase::dependencies::extract::extract_import_facts_from_program;
use crate::codebase::ts_source::facts::{self, TsFileFacts};
use crate::codebase::ts_symbols::extract_symbols_from_program;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::Path;

pub(crate) fn collect_file_facts(
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
) -> Option<CheckFileFacts> {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => {
            return Some(CheckFileFacts {
                parse_error: Some(format!("failed to read {}: {err}", path.display())),
                ..CheckFileFacts::default()
            });
        }
    };
    if plan.storybook && path.extension().and_then(|ext| ext.to_str()) == Some("mdx") {
        let stored_source = should_store_source(plan).then_some(source.clone());
        return Some(CheckFileFacts {
            ts: ts_source(stored_source.clone()),
            source: stored_source,
            storybook: Some(crate::codebase::storybook::extract_mdx_source(&source)),
            ..CheckFileFacts::default()
        });
    }
    if plan.raw_source && !requires_parse(plan, path, playwright) {
        return Some(CheckFileFacts {
            ts: ts_source(Some(source.clone())),
            source: Some(source),
            ..CheckFileFacts::default()
        });
    }
    if !requires_parse(plan, path, playwright) {
        return Some(CheckFileFacts::default());
    }
    let source_type = match SourceType::from_path(path) {
        Ok(source_type) => source_type,
        Err(_) => {
            let stored_source = should_store_source(plan).then_some(source);
            return Some(CheckFileFacts {
                ts: ts_source(stored_source.clone()),
                source: stored_source,
                parse_error: Some(format!("unsupported file type: {}", path.display())),
                ..CheckFileFacts::default()
            });
        }
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, source_type).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        let parse_error = parsed
            .errors
            .first()
            .map(|error| format!("{error:?}"))
            .unwrap_or("parser panicked without diagnostic details".to_string());
        let stored_source = should_store_source(plan).then_some(source.clone());
        let ts = super::file_parse_error::ts_facts(plan, stored_source.clone(), &parsed.program);
        return Some(CheckFileFacts {
            ts,
            source: stored_source,
            parse_error: Some(parse_error),
            parsed: true,
            ..CheckFileFacts::default()
        });
    }
    let program = &parsed.program;
    let needs_import_facts = plan.imports || plan.graph.imports || plan.graph.function_calls;
    let import_facts = if needs_import_facts {
        extract_import_facts_from_program(program)
    } else {
        Default::default()
    };
    let symbols = if plan.symbols || plan.graph.symbols {
        Some(extract_symbols_from_program(program, &source))
    } else {
        None
    };
    let react = if plan.react || plan.graph.react {
        Some(crate::react_traits::analyze::file::analyze_program(
            path, root, &source, program,
        ))
    } else {
        None
    };
    let queue = if plan.queue || plan.graph.queue_project {
        Some(crate::queue::extract::extract_program_with_factories(
            path,
            &source,
            program,
            &plan.queue_factory_names,
        ))
    } else {
        None
    };
    let integration = plan
        .integration
        .then(|| crate::integration_tests::analysis::analyze_program(path, program, &source));
    let dynamic_imports = if plan.dynamic_imports {
        Some(
            crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::extract_program(
                &source, program,
            ),
        )
    } else {
        None
    };
    let nextjs_caching = plan.nextjs_caching.then(|| {
        crate::codebase::rules::nextjs_no_caching::extract_program(path, &source, program)
    });
    let storybook = plan
        .storybook
        .then(|| crate::codebase::storybook::extract_program(&source, program));
    let domain = if plan.graph.has_domain_facts() {
        facts::domain::collect_domain_facts(program, path, &source, plan.graph, &plan.graph_context)
    } else {
        facts::domain::DomainFacts::default()
    };
    let queue_project = queue.or(domain.queue_project);
    let playwright =
        super::file_playwright::collect_playwright_facts(path, program, &source, playwright);
    let ts = TsFileFacts {
        source: should_store_source(plan).then_some(source.clone()),
        imports: import_facts.imports,
        function_calls: import_facts.function_calls,
        symbol_references: import_facts.symbol_references,
        local_type_declarations: Default::default(),
        exported_functions: import_facts.exported_functions,
        unknown_callers: import_facts.unknown_callers,
        has_unknown_top_level_call: import_facts.has_unknown_top_level_call,
        symbols: symbols.clone(),
        route_refs: domain.route_refs,
        backend_routes: domain.backend_routes,
        queue_usage: domain.queue_usage,
        queue_create_line: domain.queue_create_line,
        queue_name: domain.queue_name,
        queue_project,
        http_calls: domain.http_calls,
        process_spawns: domain.process_spawns,
        server_routes: domain.server_routes,
        react_components: react
            .as_ref()
            .map(|analysis| analysis.components.clone())
            .unwrap_or_default(),
    };
    Some(CheckFileFacts {
        ts,
        source: should_store_source(plan).then_some(source),
        symbols,
        react,
        integration,
        dynamic_imports,
        nextjs_caching,
        storybook,
        playwright,
        parse_error: None,
        parsed: true,
    })
}

fn should_store_source(plan: &CheckFactPlan) -> bool {
    plan.source || plan.raw_source
}

fn ts_source(source: Option<String>) -> TsFileFacts {
    TsFileFacts {
        source,
        ..Default::default()
    }
}

fn requires_parse(
    plan: &CheckFactPlan,
    path: &Path,
    playwright: Option<&PlaywrightFactPlan>,
) -> bool {
    plan.imports
        || plan.symbols
        || plan.react
        || plan.queue
        || plan.integration
        || plan.dynamic_imports
        || plan.nextjs_caching
        || plan.storybook
        || !plan.graph.is_empty()
        || playwright.is_some_and(|plan| plan.test_id_attributes_by_path.contains_key(path))
        || plan.source
        || (!plan.raw_source && playwright.is_none())
}
