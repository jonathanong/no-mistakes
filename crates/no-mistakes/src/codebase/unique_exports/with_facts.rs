use super::{analyze_unique_exports, filter_source_files, load_codebase_config_with_path};
use super::{normalize_path, workspaces};
use super::{
    ImportResolver, PreparedUniqueExportFinding, UniqueExportFinding, UniqueExportsOptions,
};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactMap;
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

mod helpers;
mod prepared;
use helpers::{filter_application_files, shared_symbol_files};
pub use prepared::{
    analyze_project_with_config_and_facts, analyze_project_with_prepared_facts,
    analyze_project_with_prepared_facts_and_inferred,
    analyze_project_with_prepared_facts_and_inferred_and_session,
    analyze_project_with_prepared_facts_catalog_and_inferred_and_session,
    analyze_project_with_prepared_facts_catalog_and_inferred_and_session_for_check,
};

pub fn analyze_project_with_facts(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
    shared: &CheckFactMap,
) -> Result<Vec<UniqueExportFinding>> {
    let root = normalize_path(root);
    let root = root.as_path();
    let config = load_codebase_config_with_path(root, config_path)?;
    analyze_project_with_config_and_facts(root, &config, tsconfig_path, shared)
}

struct ProjectRootsAnalysis<'a> {
    session: &'a AnalysisSession,
    root: &'a Path,
    application_filter: Option<(
        &'a crate::codebase::config::Config,
        &'a crate::codebase::config::RuleApplicationConfig,
    )>,
    resolution: prepared::PreparedResolution<'a>,
    shared: &'a CheckFactMap,
    project_roots: Vec<std::path::PathBuf>,
    options: UniqueExportsOptions,
    defer_suppression: bool,
    inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
    config: &'a crate::codebase::config::Config,
}

fn analyze_project_roots_with_facts(
    inputs: ProjectRootsAnalysis<'_>,
) -> Result<Vec<PreparedUniqueExportFinding>> {
    let ProjectRootsAnalysis {
        session,
        root,
        application_filter,
        resolution,
        shared,
        project_roots,
        options,
        defer_suppression,
        inferred_roots,
        config,
    } = inputs;
    if project_roots.is_empty() {
        return Ok(Vec::new());
    }
    let workspace_files = shared.files().to_vec();
    let mut analysis_files = workspace_files
        .iter()
        .filter(|path| {
            project_roots
                .iter()
                .any(|project_root| path.starts_with(project_root))
        })
        .cloned()
        .collect::<Vec<_>>();
    if let Some((config, application)) = application_filter {
        analysis_files =
            filter_application_files(root, config, application, analysis_files, inferred_roots)?;
    }
    analysis_files.sort();
    analysis_files.dedup();
    let analysis_files = filter_source_files(&analysis_files);
    let symbol_files = shared_symbol_files(&workspace_files, &analysis_files);
    let loaded_tsconfig = (resolution.tsconfig.is_none() && resolution.catalog.is_none())
        .then(|| {
            crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
                resolution.tsconfig_path,
                root,
                shared.files(),
            )
        })
        .transpose()?;
    let visible_files = workspace_files
        .iter()
        .map(|path| normalize_path(path))
        .collect::<HashSet<_>>();
    let workspace = workspaces::load_from_files_with_session(root, &workspace_files, Some(session))
        .unwrap_or_default();
    let remix_roots = super::remix::configured_roots(root, config, inferred_roots);
    let source_files = super::scan::collect_source_files_from_facts_with_sources(
        root,
        &symbol_files,
        shared,
        defer_suppression,
        session.existing_sources_for(root).as_deref(),
        &remix_roots,
    )?;
    if let Some(catalog) = resolution.catalog {
        let resolver = crate::codebase::ts_resolver::ScopedImportResolver::new_in_session(
            catalog,
            &visible_files,
            session,
        );
        analyze_unique_exports(
            root,
            analysis_files,
            source_files,
            options,
            resolver,
            workspace,
        )
    } else {
        let tsconfig = resolution
            .tsconfig
            .or(loaded_tsconfig.as_ref())
            .expect("prepared or locally resolved tsconfig");
        let resolver = ImportResolver::new_in_session(tsconfig, Some(&visible_files), session);
        analyze_unique_exports(
            root,
            analysis_files,
            source_files,
            options,
            resolver,
            workspace,
        )
    }
}

#[cfg(test)]
mod tests;
