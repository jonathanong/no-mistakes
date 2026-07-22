use crate::queue::extract::FileFacts;
use crate::queue::graph_build::{build_prepared_report, build_report};
use crate::queue::graph_model::PreparedProjectReport;
use crate::queue::graph_model::{build_filter, InternalProducer, InternalWorker, ProjectReport};
use crate::queue::graph_resolution::{queue_definitions, resolve_producers, resolve_workers};
use crate::queue::resolver::{load_tsconfig_from_visible, queue_import_resolver};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod catalog;
mod fact_sources;
mod standalone;
use fact_sources::{queue_project_facts_from_shared, queue_project_facts_from_ts};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RelatedDirection {
    Deps,
    Dependents,
    Both,
}

pub fn analyze_project(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
) -> anyhow::Result<ProjectReport> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    standalone::analyze_project(&root, tsconfig_path, filters, snapshot)
}

#[doc(hidden)]
pub fn analyze_project_indexed(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
) -> anyhow::Result<PreparedProjectReport> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    standalone::analyze_project_indexed(&root, tsconfig_path, filters, snapshot)
}

pub fn analyze_project_with_facts(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> anyhow::Result<ProjectReport> {
    analyze_project_with_facts_inner(root, tsconfig_path, filters, shared, build_report)
}

fn analyze_project_with_facts_inner<T>(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
    builder: impl FnOnce(
        &Path,
        Vec<InternalProducer>,
        Vec<InternalWorker>,
        &HashMap<PathBuf, FileFacts>,
    ) -> T,
) -> anyhow::Result<T> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let root = root.as_path();
    let tsconfig = load_tsconfig_from_visible(root, tsconfig_path, shared.files())?;
    let filter = build_filter(filters)?;
    Ok(resolve_queue_relationships(
        root,
        &tsconfig,
        filter.as_ref(),
        shared,
        builder,
    ))
}

/// Analyze shared queue facts with a TypeScript config already prepared by the caller.
///
/// Legacy callers use this entrypoint with one caller-selected config. Aggregate
/// checks use the catalog-aware entrypoint below so each importer selects its
/// visible workspace config.
#[doc(hidden)]
pub fn analyze_project_with_prepared_facts(
    root: &Path,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> anyhow::Result<ProjectReport> {
    analyze_project_with_prepared_facts_inner(root, tsconfig, filters, shared, build_report)
}

/// Analyze aggregate queue facts with per-importer workspace TypeScript resolution.
///
/// This keeps the legacy prepared-config API deterministic for explicit callers,
/// while aggregate checks select aliases from the importing package's catalog entry.
#[doc(hidden)]
pub fn analyze_project_with_prepared_facts_and_catalog_and_session(
    root: &Path,
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> anyhow::Result<ProjectReport> {
    catalog::analyze(root, tsconfig_catalog, filters, shared, session)
}

#[doc(hidden)]
pub fn analyze_project_with_prepared_facts_indexed_and_catalog_and_session(
    root: &Path,
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> anyhow::Result<PreparedProjectReport> {
    catalog::analyze_with(
        root,
        tsconfig_catalog,
        filters,
        shared,
        session,
        build_prepared_report,
    )
}

#[doc(hidden)]
pub fn analyze_project_with_prepared_facts_indexed(
    root: &Path,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> anyhow::Result<PreparedProjectReport> {
    analyze_project_with_prepared_facts_inner(
        root,
        tsconfig,
        filters,
        shared,
        build_prepared_report,
    )
}

fn analyze_project_with_prepared_facts_inner<T>(
    root: &Path,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
    builder: impl FnOnce(
        &Path,
        Vec<InternalProducer>,
        Vec<InternalWorker>,
        &HashMap<PathBuf, FileFacts>,
    ) -> T,
) -> anyhow::Result<T> {
    let root = root.canonicalize().unwrap_or(root.to_path_buf());
    let root = root.as_path();
    let filter = build_filter(filters)?;
    Ok(resolve_queue_relationships(
        root,
        tsconfig,
        filter.as_ref(),
        shared,
        builder,
    ))
}

fn resolve_queue_relationships<T>(
    root: &Path,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    filter: Option<&globset::GlobSet>,
    shared: &crate::codebase::check_facts::CheckFactMap,
    builder: impl FnOnce(
        &Path,
        Vec<InternalProducer>,
        Vec<InternalWorker>,
        &HashMap<PathBuf, FileFacts>,
    ) -> T,
) -> T {
    let facts = queue_project_facts_from_shared(shared, filter, root);
    let visible_files = shared.files().iter().cloned().collect();
    let resolver = queue_import_resolver(tsconfig, root, &visible_files);
    resolve_queue_relationships_with_resolver(root, &facts, &resolver, builder)
}

pub(super) fn resolve_queue_relationships_with_resolver<
    T,
    R: crate::codebase::ts_resolver::ImportResolverFacade,
>(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &R,
    builder: impl FnOnce(
        &Path,
        Vec<InternalProducer>,
        Vec<InternalWorker>,
        &HashMap<PathBuf, FileFacts>,
    ) -> T,
) -> T {
    let queue_defs = queue_definitions(facts);
    let remapper =
        crate::codebase::ts_source::FrozenPathRemapper::from_paths(facts.keys().cloned());
    let producers = resolve_producers(facts, &queue_defs, resolver, &remapper);
    let workers = resolve_workers(facts, &queue_defs, resolver, &remapper);
    builder(root, producers, workers, facts)
}
