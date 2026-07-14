use super::{analyze_unique_exports, filter_source_files, load_codebase_config_with_path};
use super::{normalize_path, workspaces};
use super::{ImportResolver, UniqueExportFinding, UniqueExportsOptions};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactMap;
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

mod prepared;
pub use prepared::analyze_project_with_prepared_facts_and_inferred_and_session;
pub use prepared::{
    analyze_project_with_config_and_facts, analyze_project_with_prepared_facts,
    analyze_project_with_prepared_facts_and_inferred,
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
    tsconfig_path: Option<&'a Path>,
    prepared_tsconfig: Option<&'a crate::codebase::ts_resolver::TsConfig>,
    shared: &'a CheckFactMap,
    project_roots: Vec<std::path::PathBuf>,
    options: UniqueExportsOptions,
    inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
}

fn analyze_project_roots_with_facts(
    inputs: ProjectRootsAnalysis<'_>,
) -> Result<Vec<UniqueExportFinding>> {
    let ProjectRootsAnalysis {
        session,
        root,
        application_filter,
        tsconfig_path,
        prepared_tsconfig,
        shared,
        project_roots,
        options,
        inferred_roots,
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
    let loaded_tsconfig = prepared_tsconfig
        .is_none()
        .then(|| {
            crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
                tsconfig_path,
                root,
                shared.files(),
            )
        })
        .transpose()?;
    let tsconfig = prepared_tsconfig
        .or(loaded_tsconfig.as_ref())
        .expect("prepared or locally resolved tsconfig");
    let visible_files = workspace_files
        .iter()
        .map(|path| normalize_path(path))
        .collect::<HashSet<_>>();
    let resolver = ImportResolver::new_in_session(tsconfig, Some(&visible_files), session);
    let workspace = workspaces::load_from_files_with_session(root, &workspace_files, Some(session))
        .unwrap_or_default();
    let source_files = super::scan::collect_source_files_from_facts(root, &symbol_files, shared)?;
    analyze_unique_exports(
        root,
        analysis_files,
        source_files,
        options,
        resolver,
        workspace,
    )
}

fn filter_application_files(
    root: &Path,
    config: &crate::codebase::config::Config,
    application: &crate::codebase::config::RuleApplicationConfig,
    files: Vec<std::path::PathBuf>,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Result<Vec<std::path::PathBuf>> {
    use crate::codebase::rules::path_filter::GlobMatcher;

    let include = GlobMatcher::new(&application.include, "unique-exports rule include")?;
    let exclude = GlobMatcher::new(&application.exclude, "unique-exports rule exclude")?;
    let mut inferred_roots = inferred_roots.cloned().unwrap_or_default();
    let projects = application
        .projects
        .iter()
        .filter_map(|project_name| {
            let project = config.projects.get(project_name)?;
            let project_root = project
                .effective_root_with_cache(root, &mut inferred_roots)
                .unwrap_or_else(|| root.to_path_buf());
            let project_root = normalize_path(&project_root);
            let project_include =
                GlobMatcher::new(&project.include, "unique-exports project include").ok()?;
            let project_exclude =
                GlobMatcher::new(&project.exclude, "unique-exports project exclude").ok()?;
            Some(ApplicationProjectFilter {
                root: project_root,
                include: project_include,
                exclude: project_exclude,
            })
        })
        .collect::<Vec<_>>();
    Ok(files
        .into_iter()
        .filter(|path| {
            let repo_rel = relative(root, path);
            let rule_match =
                (include.is_empty() || include.is_match(&repo_rel)) && !exclude.is_match(&repo_rel);
            if application.repository && rule_match {
                return true;
            }
            projects.iter().any(|project| {
                if !path.starts_with(&project.root) {
                    return false;
                }
                let project_rel = relative(&project.root, path);
                (project.include.is_empty()
                    || project.include.is_match(&repo_rel)
                    || project.include.is_match(&project_rel))
                    && !project.exclude.is_match(&repo_rel)
                    && !project.exclude.is_match(&project_rel)
                    && (include.is_empty()
                        || include.is_match(&repo_rel)
                        || include.is_match(&project_rel))
                    && !exclude.is_match(&repo_rel)
                    && !exclude.is_match(&project_rel)
            })
        })
        .collect())
}

struct ApplicationProjectFilter {
    root: std::path::PathBuf,
    include: crate::codebase::rules::path_filter::GlobMatcher,
    exclude: crate::codebase::rules::path_filter::GlobMatcher,
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests;

fn shared_symbol_files(
    workspace_files: &[std::path::PathBuf],
    analysis_files: &[std::path::PathBuf],
) -> Vec<std::path::PathBuf> {
    let mut symbol_files = workspace_files.to_vec();
    symbol_files.extend(analysis_files.iter().cloned());
    symbol_files.sort();
    symbol_files.dedup();
    filter_source_files(&symbol_files)
}
