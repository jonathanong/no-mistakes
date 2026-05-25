use super::{analyze_unique_exports, filter_source_files, load_codebase_config_with_path};
use super::{find_tsconfig, load_tsconfig, normalize_path, workspaces, DefaultTsConfig};
use super::{ImportResolver, UniqueExportFinding, UniqueExportsOptions, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use anyhow::Result;
use std::path::Path;

pub fn analyze_project_with_facts(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
    shared: &CheckFactMap,
) -> Result<Vec<UniqueExportFinding>> {
    let root = normalize_path(root);
    let root = root.as_path();
    let config = load_codebase_config_with_path(root, config_path)?;
    let applications = config.rule_applications_for(RULE_ID);
    if !applications.is_empty() {
        let mut findings = Vec::new();
        for application in applications {
            let project_roots = config
                .project_roots_for_rule_application(root, application)
                .into_iter()
                .map(|path| normalize_path(&path))
                .collect::<Vec<_>>();
            let options = application.rule_options();
            let application_findings = analyze_project_roots_with_facts(
                root,
                Some((&config, application)),
                tsconfig_path,
                shared,
                project_roots,
                options,
            )?;
            findings.extend(application_findings);
        }
        findings.sort();
        findings.dedup();
        return Ok(findings);
    }
    let project_roots = config
        .project_roots_for_rule(root, RULE_ID)
        .into_iter()
        .map(|path| normalize_path(&path))
        .collect::<Vec<_>>();
    analyze_project_roots_with_facts(
        root,
        None,
        tsconfig_path,
        shared,
        project_roots,
        config.rule_options(RULE_ID),
    )
}

fn analyze_project_roots_with_facts(
    root: &Path,
    application_filter: Option<(
        &crate::codebase::config::Config,
        &crate::codebase::config::RuleApplicationConfig,
    )>,
    tsconfig_path: Option<&Path>,
    shared: &CheckFactMap,
    project_roots: Vec<std::path::PathBuf>,
    options: UniqueExportsOptions,
) -> Result<Vec<UniqueExportFinding>> {
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
        analysis_files = filter_application_files(root, config, application, analysis_files)?;
    }
    analysis_files.sort();
    analysis_files.dedup();
    let analysis_files = filter_source_files(&analysis_files);
    let symbol_files = shared_symbol_files(&workspace_files, &analysis_files);
    let tsconfig = match tsconfig_path {
        Some(path) => {
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            load_tsconfig(&path)?
        }
        None => find_tsconfig(root)
            .map(|path| load_tsconfig(&path))
            .transpose()?
            .unwrap_or_default_for(root),
    };
    let resolver = ImportResolver::new(&tsconfig);
    let workspace = workspaces::load_from_files(root, &workspace_files).unwrap_or_default();
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
) -> Result<Vec<std::path::PathBuf>> {
    use crate::codebase::rules::path_filter::GlobMatcher;

    let include = GlobMatcher::new(&application.include, "unique-exports rule include")?;
    let exclude = GlobMatcher::new(&application.exclude, "unique-exports rule exclude")?;
    let projects = application
        .projects
        .iter()
        .filter_map(|project_name| {
            let project = config.projects.get(project_name)?;
            let project_root = project
                .effective_root(root)
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
