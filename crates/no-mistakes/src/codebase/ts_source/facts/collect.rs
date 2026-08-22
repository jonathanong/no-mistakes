use super::{TsFactContext, TsFactMap, TsFactPlan};
use crate::codebase::dependencies::extract::is_indexable;
use rayon::prelude::*;
use std::path::PathBuf;

mod file;
pub(crate) use file::collect_file_facts_from_program;
use file::collect_file_facts_with_sources_and_session;

#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod parse_cache_tests;

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
    let sources = session
        .existing_sources_for(&context.root)
        .unwrap_or_else(|| {
            std::sync::Arc::new(crate::codebase::ts_source::SourceStore::new_observed(
                std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(files)),
                session.observer().cloned(),
            ))
        });
    collect_ts_facts_with_context_sources_and_session(session, files, plan, context, &sources)
}

pub(crate) fn collect_ts_facts_with_context_sources_and_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    files: &[PathBuf],
    plan: TsFactPlan,
    context: &TsFactContext,
    sources: &crate::codebase::ts_source::SourceStore,
) -> TsFactMap {
    collect_ts_facts_with_context_sources_and_session_serializing_paths(
        session,
        files,
        plan,
        context,
        sources,
        &[],
    )
}

/// Collect one project-wide fact map, keeping selected paths on the calling
/// thread so they can reuse parser state produced by a serial preparatory
/// phase (such as runner-config analysis). All other files remain parallel.
pub(crate) fn collect_ts_facts_with_context_sources_and_session_serializing_paths(
    session: &crate::codebase::analysis_session::AnalysisSession,
    files: &[PathBuf],
    plan: TsFactPlan,
    context: &TsFactContext,
    sources: &crate::codebase::ts_source::SourceStore,
    serial_paths: &[PathBuf],
) -> TsFactMap {
    let files = crate::codebase::ts_source::deduplicate_analysis_paths(
        files.iter().filter(|path| is_indexable(path)),
    );
    // Count fact extraction separately from physical parses: a request-local
    // parser cache can hide duplicate Rayon collection passes on different
    // workers, while this records every file handed to the fact collector.
    session.record_work("ts_facts.collections", 1);
    session.record_work("ts_facts.files", files.len() as u64);
    if serial_paths.is_empty() {
        return TsFactMap::from_iter_with_plan_and_inventory(
            files
                .par_iter()
                .map(|path| {
                    crate::invocation::check_timeout().ok().map(|()| {
                        collect_file_facts_with_sources_and_session(
                            session, path, plan, context, sources, false,
                        )
                        .map(|facts| (path.clone(), facts))
                    })
                })
                .while_some()
                .flatten()
                .collect::<Vec<_>>(),
            plan,
            std::sync::Arc::clone(sources.inventory()),
        );
    }
    let serial_paths = serial_paths
        .iter()
        .collect::<std::collections::HashSet<_>>();
    let (serial_files, parallel_files): (Vec<_>, Vec<_>) = files
        .into_iter()
        .partition(|path| serial_paths.contains(path));
    let mut facts = serial_files
        .iter()
        .filter_map(|path| {
            crate::invocation::check_timeout()
                .ok()
                .and_then(|()| {
                    collect_file_facts_with_sources_and_session(
                        session, path, plan, context, sources, true,
                    )
                })
                .map(|facts| (path.clone(), facts))
        })
        .collect::<Vec<_>>();
    for path in &serial_files {
        crate::ast::evict_request_parse_cache_path(path);
    }
    facts.extend(
        parallel_files
            .par_iter()
            .map(|path| {
                crate::invocation::check_timeout().ok().map(|()| {
                    collect_file_facts_with_sources_and_session(
                        session, path, plan, context, sources, false,
                    )
                    .map(|facts| (path.clone(), facts))
                })
            })
            .while_some()
            .flatten()
            .collect::<Vec<_>>(),
    );
    TsFactMap::from_iter_with_plan_and_inventory(
        facts,
        plan,
        std::sync::Arc::clone(sources.inventory()),
    )
}
