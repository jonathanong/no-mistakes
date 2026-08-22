use super::filter_source_files;
use crate::codebase::config::{Config, InferredRoots, RuleApplicationConfig};
use crate::codebase::rules::path_filter::GlobMatcher;
use crate::codebase::ts_resolver::normalize_path;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) struct ApplicationProjectFilter {
    pub(super) root: PathBuf,
    pub(super) include: GlobMatcher,
    pub(super) exclude: GlobMatcher,
}

pub(super) fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(super) fn shared_symbol_files(
    workspace_files: &[PathBuf],
    analysis_files: &[PathBuf],
) -> Vec<PathBuf> {
    let mut symbol_files = workspace_files.to_vec();
    symbol_files.extend(analysis_files.iter().cloned());
    symbol_files.sort();
    symbol_files.dedup();
    filter_source_files(&symbol_files)
}

pub(super) fn filter_application_files(
    root: &Path,
    config: &Config,
    application: &RuleApplicationConfig,
    files: Vec<PathBuf>,
    inferred_roots: Option<&InferredRoots>,
) -> Result<Vec<PathBuf>> {
    let include = GlobMatcher::new(&application.include, "unique-exports rule include")?;
    let exclude = GlobMatcher::new(&application.exclude, "unique-exports rule exclude")?;
    let mut inferred_roots = inferred_roots.cloned().unwrap_or_default();
    let mut projects = Vec::new();
    for project_name in &application.projects {
        let Some(project) = config.projects.get(project_name) else {
            continue;
        };
        let project_root = match project.effective_root_with_cache(root, &mut inferred_roots) {
            Some(project_root) => project_root,
            None => root.to_path_buf(),
        };
        let project_root = normalize_path(&project_root);
        let Ok(project_include) =
            GlobMatcher::new(&project.include, "unique-exports project include")
        else {
            continue;
        };
        let Ok(project_exclude) =
            GlobMatcher::new(&project.exclude, "unique-exports project exclude")
        else {
            continue;
        };
        projects.push(ApplicationProjectFilter {
            root: project_root,
            include: project_include,
            exclude: project_exclude,
        });
    }
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
