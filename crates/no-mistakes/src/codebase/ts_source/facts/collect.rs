use super::{domain, TsFactContext, TsFactMap, TsFactPlan, TsFileFacts};
use crate::codebase::dependencies::extract::{
    extract_import_facts_from_program_with_source, is_indexable,
};
use crate::codebase::ts_symbols::extract_symbols_from_program;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub fn collect_ts_facts(files: &[PathBuf], plan: TsFactPlan) -> TsFactMap {
    assert!(
        !plan.has_domain_facts(),
        "domain fact plans require collect_ts_facts_with_context"
    );
    collect_ts_facts_with_context(files, plan, &TsFactContext::default())
}

pub fn collect_ts_facts_with_context(
    files: &[PathBuf],
    plan: TsFactPlan,
    context: &TsFactContext,
) -> TsFactMap {
    let facts = files
        .par_iter()
        .filter(|path| is_indexable(path))
        .filter_map(|path| {
            collect_file_facts(path, plan, context).map(|facts| (path.clone(), facts))
        })
        .collect();
    TsFactMap::with_plan(facts, plan)
}

pub(crate) fn collect_file_facts(
    path: &Path,
    plan: TsFactPlan,
    context: &TsFactContext,
) -> Option<TsFileFacts> {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            return Some(TsFileFacts {
                parse_error: Some(format!("failed to read {}: {error}", path.display())),
                ..TsFileFacts::default()
            });
        }
    };
    #[cfg(any(test, feature = "test-instrumentation"))]
    crate::ast::record_parse_path(path);
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
    let parsed = Parser::new(&allocator, &source, source_type).parse();
    let parse_error = if parsed.panicked || !parsed.diagnostics.is_empty() {
        Some(crate::codebase::ts_source::format_parse_diagnostic(
            path,
            &parsed.diagnostics,
        ))
    } else {
        None
    };
    Some(collect_file_facts_from_program(
        path,
        plan,
        context,
        &source,
        &parsed.program,
        parse_error,
    ))
}

pub(crate) fn collect_file_facts_from_program(
    path: &Path,
    plan: TsFactPlan,
    context: &TsFactContext,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    parse_error: Option<String>,
) -> TsFileFacts {
    let import_facts = if plan.imports || plan.function_calls {
        extract_import_facts_from_program_with_source(program, source)
    } else {
        Default::default()
    };
    let symbols = plan
        .symbols
        .then(|| extract_symbols_from_program(program, source));
    let domain = if plan.has_domain_facts() {
        domain::collect_domain_facts(program, path, source, plan, context)
    } else {
        domain::DomainFacts::default()
    };
    let react_components = if plan.react {
        match context.visible_files.as_deref() {
            Some(visible) => crate::react_traits::analyze::file::analyze_program_from_visible(
                path,
                &context.root,
                source,
                program,
                visible,
            ),
            None => crate::react_traits::analyze::file::analyze_program(
                path,
                &context.root,
                source,
                program,
            ),
        }
        .components
    } else {
        Vec::new()
    };
    TsFileFacts {
        parse_error,
        source: plan.source.then(|| source.to_string()),
        imports: import_facts.imports,
        function_calls: import_facts.function_calls,
        symbol_references: import_facts.symbol_references,
        exported_functions: import_facts.exported_functions,
        unknown_callers: import_facts.unknown_callers,
        has_unknown_top_level_call: import_facts.has_unknown_top_level_call,
        symbols,
        route_refs: domain.route_refs,
        route_helpers: domain.route_helpers,
        route_helper_imports: domain.route_helper_imports,
        route_helper_refs: domain.route_helper_refs,
        backend_routes: domain.backend_routes,
        queue_usage: domain.queue_usage,
        queue_create_line: domain.queue_create_line,
        queue_name: domain.queue_name,
        queue_project: domain.queue_project,
        http_calls: domain.http_calls,
        process_spawns: domain.process_spawns,
        server_routes: domain.server_routes,
        react_components,
        effect_calls: domain.effect_calls,
        rsc_environment: domain.rsc_environment,
    }
}
