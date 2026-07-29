use super::super::{domain, TsFactContext, TsFactPlan, TsFileFacts};
use crate::codebase::dependencies::extract::{
    extract_import_facts_from_program_with_source_and_resource_roots, is_indexable,
};
use crate::codebase::ts_symbols::extract_symbols_from_program;
use std::path::Path;

pub(super) fn facts_from_collection_result(result: anyhow::Result<TsFileFacts>) -> TsFileFacts {
    match result {
        Ok(facts) => facts,
        // Indexable extensions are expected to have an OXC source type.
        // Reaching this branch means that allowlist and OXC support drifted.
        Err(error) => TsFileFacts {
            parse_error: Some(error.to_string()),
            ..TsFileFacts::default()
        },
    }
}

pub(super) fn collect_file_facts_with_sources_and_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    path: &Path,
    plan: TsFactPlan,
    context: &TsFactContext,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Option<TsFileFacts> {
    let source = match sources.read_path(path) {
        Ok(source) => source,
        Err(error) => {
            return Some(TsFileFacts {
                parse_error: Some(format!("failed to read {}: {error}", path.display())),
                ..TsFileFacts::default()
            });
        }
    };
    // Project-wide collection filters to known TS/JS extensions. Parse those
    // in standard recovered mode so runner-config analysis can share the same
    // request cache. The direct-file test/support path retains the historical
    // TypeScript fallback for unknown extensions.
    let result = if is_indexable(path) {
        session.with_recovered_program_status(
            path,
            &source,
            |program, source, parse_error, panicked| {
                let mut facts = collect_file_facts_from_program(
                    path,
                    plan,
                    context,
                    source,
                    program,
                    parse_error,
                );
                facts.fatal_parse_error = panicked;
                facts
            },
        )
    } else {
        session.with_recovered_typescript_program(path, &source, |program, source, parse_error| {
            collect_file_facts_from_program(path, plan, context, source, program, parse_error)
        })
    };
    Some(facts_from_collection_result(result))
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
        extract_import_facts_from_program_with_source_and_resource_roots(
            program,
            source,
            plan.resources,
        )
    } else {
        Default::default()
    };
    let resources = if plan.resources {
        crate::codebase::ts_resources::extract(program, source)
    } else {
        Default::default()
    };
    let symbols = plan
        .symbols
        .then(|| extract_symbols_from_program(program, source));
    let call_sites = if plan.call_sites {
        super::super::call_sites::collect_call_site_facts(program, source)
    } else {
        Vec::new()
    };
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
        Default::default()
    };
    TsFileFacts {
        parse_error,
        fatal_parse_error: false,
        source: plan.source.then(|| source.to_owned()),
        imports: import_facts.imports,
        function_calls: import_facts.function_calls,
        call_sites,
        resource_calls: resources.calls,
        resource_diagnostics: resources.diagnostics,
        symbol_references: import_facts.symbol_references,
        exported_functions: import_facts.exported_functions,
        exported_resource_roots: import_facts.exported_resource_roots,
        exported_resource_scopes: import_facts.exported_resource_scopes,
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
        react_components: react_components.as_ref().clone(),
        effect_calls: domain.effect_calls,
        rsc_environment: domain.rsc_environment,
    }
}
