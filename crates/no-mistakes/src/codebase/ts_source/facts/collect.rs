use super::{domain, TsFactContext, TsFactMap, TsFactPlan, TsFileFacts};
use crate::codebase::dependencies::extract::{
    extract_import_facts_from_program_with_source, is_indexable,
};
use crate::codebase::ts_symbols::extract_symbols_from_program;
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
    let inventory =
        std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(files));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    collect_ts_facts_with_context_and_sources(files, plan, context, &sources)
}

#[doc(hidden)]
pub fn collect_ts_facts_with_context_and_sources(
    files: &[PathBuf],
    plan: TsFactPlan,
    context: &TsFactContext,
    sources: &crate::codebase::ts_source::SourceStore,
) -> TsFactMap {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    collect_ts_facts_with_context_sources_and_session(&session, files, plan, context, sources)
}

#[doc(hidden)]
pub fn collect_ts_facts_with_session_and_context(
    session: &crate::codebase::analysis_session::AnalysisSession,
    files: &[PathBuf],
    plan: TsFactPlan,
    context: &TsFactContext,
) -> TsFactMap {
    let inventory =
        std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(files));
    let sources = crate::codebase::ts_source::SourceStore::new_observed(
        inventory,
        session.observer().cloned(),
    );
    collect_ts_facts_with_context_sources_and_session(session, files, plan, context, &sources)
}

pub(crate) fn collect_ts_facts_with_context_sources_and_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    files: &[PathBuf],
    plan: TsFactPlan,
    context: &TsFactContext,
    sources: &crate::codebase::ts_source::SourceStore,
) -> TsFactMap {
    let files = crate::codebase::ts_source::deduplicate_analysis_paths(
        files.iter().filter(|path| is_indexable(path)),
    );
    let facts = files
        .par_iter()
        .filter_map(|path| {
            collect_file_facts_with_sources_and_session(session, path, plan, context, sources)
                .map(|facts| (path.clone(), facts))
        })
        .collect();
    TsFactMap::with_plan(facts, plan)
}

#[cfg(test)]
pub(crate) fn collect_file_facts_with_sources(
    path: &Path,
    plan: TsFactPlan,
    context: &TsFactContext,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Option<TsFileFacts> {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    collect_file_facts_with_sources_and_session(&session, path, plan, context, sources)
}

fn collect_file_facts_with_sources_and_session(
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
    match session.with_recovered_typescript_program(
        path,
        &source,
        |program, source, parse_error| {
            collect_file_facts_from_program(path, plan, context, source, program, parse_error)
        },
    ) {
        Ok(facts) => Some(facts),
        // This collector historically parsed unsupported extensions as TS.
        // It is only called for indexable files, so reaching this branch means
        // the extension allowlist and OXC source-type support drifted apart.
        Err(error) => Some(TsFileFacts {
            parse_error: Some(error.to_string()),
            ..TsFileFacts::default()
        }),
    }
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
        .then(|| std::sync::Arc::new(extract_symbols_from_program(program, source)));
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
        source: plan.source.then(|| source.to_owned()),
        imports: import_facts.imports,
        function_calls: import_facts.function_calls,
        symbol_references: import_facts.symbol_references,
        exported_functions: import_facts.exported_functions,
        unknown_callers: import_facts.unknown_callers,
        has_unknown_top_level_call: import_facts.has_unknown_top_level_call,
        symbols: symbols.as_deref().cloned(),
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
