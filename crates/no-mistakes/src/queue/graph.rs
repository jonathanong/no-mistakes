use crate::config::v2::NoMistakesConfig;
use crate::queue::extract::FileFacts;
use crate::queue::graph_build::{build_prepared_report, build_report};
use crate::queue::graph_model::PreparedProjectReport;
use crate::queue::graph_model::{build_filter, InternalProducer, InternalWorker, ProjectReport};
use crate::queue::resolver::{load_tsconfig_from_visible, queue_import_resolver};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod catalog;
mod fact_sources;
mod resolve;
mod standalone;
use fact_sources::{queue_project_facts_from_shared, queue_project_facts_from_ts};
pub(super) use resolve::resolve_queue_relationships_with_resolver;

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

/// Analyze shared facts with a single TypeScript resolver. For caller-selected
/// queue policy, use the catalog-aware prepared entry point.
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
    Ok(resolve_legacy_queue_relationships(
        root,
        &tsconfig,
        filter.as_ref(),
        shared,
        builder,
    ))
}

/// Analyze shared queue facts with a TypeScript config already prepared by the caller.
///
/// Aggregate checks use the catalog-aware entry point below, which accepts the
/// selected queue policy and resolves aliases with each importer's workspace config.
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
    config: &NoMistakesConfig,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> anyhow::Result<ProjectReport> {
    catalog::analyze(root, tsconfig_catalog, config, filters, shared, session)
}

#[doc(hidden)]
pub fn analyze_project_with_prepared_facts_indexed_and_catalog_and_session(
    root: &Path,
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
    config: &NoMistakesConfig,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> anyhow::Result<PreparedProjectReport> {
    catalog::analyze_with(
        root,
        tsconfig_catalog,
        config,
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
    Ok(resolve_legacy_queue_relationships(
        root,
        tsconfig,
        filter.as_ref(),
        shared,
        builder,
    ))
}

fn resolve_legacy_queue_relationships<T>(
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
    let visible_files: crate::fx::PathSet = shared.files().iter().cloned().collect();
    let base_resolver = queue_import_resolver(tsconfig, root, &visible_files);
    let workspace =
        crate::codebase::workspaces::load_indexed_from_files(root, shared.files())
            .unwrap_or_default();
    let resolver = crate::codebase::ts_resolver::WorkspaceFallbackResolver::new(
        &base_resolver,
        &workspace,
        &visible_files,
    );
    resolve_queue_relationships_with_resolver(root, &facts, &resolver, builder)
}
