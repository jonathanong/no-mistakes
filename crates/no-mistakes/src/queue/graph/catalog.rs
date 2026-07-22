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
    let visible_files = shared.files().iter().cloned().collect();
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::new_in_session(
        tsconfig_catalog,
        &visible_files,
        session,
    )
    .with_queue_compatibility(root);
    Ok(resolve_queue_relationships_with_resolver(
        root, &facts, &resolver, builder,
    ))
}
