use crate::queue::extract::FileFacts;
use crate::queue::graph_build::{build_prepared_report, build_report};
use crate::queue::graph_model::PreparedProjectReport;
use crate::queue::graph_model::{build_filter, InternalProducer, InternalWorker, ProjectReport};
use crate::queue::graph_resolution::{queue_definitions, resolve_producers, resolve_workers};
use crate::queue::resolver::{load_tsconfig_from_visible, queue_import_resolver};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod fact_sources;
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
    analyze_project_inner(&root, tsconfig_path, filters, snapshot, build_report)
}

#[doc(hidden)]
pub fn analyze_project_indexed(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
) -> anyhow::Result<PreparedProjectReport> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    analyze_project_inner(
        &root,
        tsconfig_path,
        filters,
        snapshot,
        build_prepared_report,
    )
}

fn analyze_project_inner<T>(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
    snapshot: crate::codebase::ts_source::VisiblePathSnapshot,
    builder: impl FnOnce(
        &Path,
        Vec<InternalProducer>,
        Vec<InternalWorker>,
        &HashMap<PathBuf, FileFacts>,
    ) -> T,
) -> anyhow::Result<T> {
    let visible_paths = snapshot.paths_for(root);
    let visible_files =
        crate::codebase::ts_source::discover_files_from_visible(root, &[], &visible_paths);
    let visible_set = visible_files.iter().cloned().collect();
    let tsconfig = load_tsconfig_from_visible(root, tsconfig_path, &visible_files)?;
    let filter = build_filter(filters)?;
    let factory_names = crate::config::v2::load_v2_config_from_visible(root, None, &visible_paths)
        .map(|config| config.queues.factories)
        .unwrap_or_default();
    let files = visible_files
        .into_iter()
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    crate::codebase::ts_source::TS_JS_EXTENSIONS.contains(&extension)
                })
        })
        .filter(|path| {
            filter
                .as_ref()
                .is_none_or(|f| f.is_match(path.strip_prefix(root).unwrap_or(path)))
        })
        .collect::<Vec<_>>();
    let mut context = crate::codebase::ts_source::facts::TsFactContext::new(root);
    context.queue_project_factory_names = factory_names;
    let ts_facts = crate::codebase::ts_source::facts::collect_ts_facts_with_context(
        &files,
        crate::codebase::ts_source::facts::TsFactPlan {
            queue_project: true,
            ..Default::default()
        },
        &context,
    );
    let facts = queue_project_facts_from_ts(ts_facts, filter.as_ref(), root);

    let queue_defs = queue_definitions(&facts);
    let resolver = queue_import_resolver(&tsconfig, root, &visible_set);
    let producers = resolve_producers(&facts, &queue_defs, &resolver);
    let workers = resolve_workers(&facts, &queue_defs, &resolver);
    Ok(builder(root, producers, workers, &facts))
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
    let facts = queue_project_facts_from_shared(shared, filter.as_ref(), root);
    let visible_files = shared.files().iter().cloned().collect();
    let queue_defs = queue_definitions(&facts);
    let resolver = queue_import_resolver(&tsconfig, root, &visible_files);
    let producers = resolve_producers(&facts, &queue_defs, &resolver);
    let workers = resolve_workers(&facts, &queue_defs, &resolver);
    Ok(builder(root, producers, workers, &facts))
}

/// Analyze shared queue facts with a TypeScript config already prepared by the caller.
///
/// The aggregate check uses this entrypoint so every domain shares the request's
/// gitignore-aware config selection. Standalone queue commands continue to use
/// [`analyze_project_with_facts`] and retain their existing config discovery behavior.
#[doc(hidden)]
pub fn analyze_project_with_prepared_facts(
    root: &Path,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> anyhow::Result<ProjectReport> {
    analyze_project_with_prepared_facts_inner(root, tsconfig, filters, shared, build_report)
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
    let facts = queue_project_facts_from_shared(shared, filter.as_ref(), root);
    let visible_files = shared.files().iter().cloned().collect();
    let queue_defs = queue_definitions(&facts);
    let resolver = queue_import_resolver(tsconfig, root, &visible_files);
    let producers = resolve_producers(&facts, &queue_defs, &resolver);
    let workers = resolve_workers(&facts, &queue_defs, &resolver);
    Ok(builder(root, producers, workers, &facts))
}
