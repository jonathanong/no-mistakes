use super::{build_filter, build_report, queue_project_facts_from_shared};
use super::{resolve_queue_relationships_with_resolver, ProjectReport};
use std::path::Path;

pub(super) fn analyze(
    root: &Path,
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> anyhow::Result<ProjectReport> {
    analyze_with(
        root,
        tsconfig_catalog,
        filters,
        shared,
        session,
        build_report,
    )
}

pub(super) fn analyze_with<T>(
    root: &Path,
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
    session: &crate::codebase::analysis_session::AnalysisSession,
    builder: impl FnOnce(
        &Path,
        Vec<super::InternalProducer>,
        Vec<super::InternalWorker>,
        &std::collections::HashMap<std::path::PathBuf, super::FileFacts>,
    ) -> T,
) -> anyhow::Result<T> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let root = root.as_path();
    let filter = build_filter(filters)?;
    let facts = queue_project_facts_from_shared(shared, filter.as_ref(), root);
    let visible_files: crate::fx::PathSet = shared.files().iter().cloned().collect();
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::new_in_session(
        tsconfig_catalog,
        &visible_files,
        session,
    )
    .with_queue_compatibility(root);
    let report = resolve_queue_relationships_with_resolver(
        root,
        &facts,
        &resolver,
        |root, mut producers, mut workers, facts| {
            if let Ok(config) = session.dataset(root).config(None) {
                let (lang_producers, lang_workers) = crate::queue::lang::language_queue_sites(
                    root,
                    session,
                    config.as_ref(),
                    filter.as_ref(),
                );
                producers.extend(lang_producers);
                workers.extend(lang_workers);
            }
            builder(root, producers, workers, facts)
        },
    );
    Ok(report)
}
