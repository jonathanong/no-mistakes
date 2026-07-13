use no_mistakes::config::v2::NoMistakesConfig;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use matching::{
    descendant_dirs_matching_suffix, descendant_dirs_matching_suffix_from_paths, GitFilesCache,
};
pub(super) use matching::{leading_globstar_literal_prefix, literal_include_prefix};

mod matching;

pub(super) fn repository_include_patterns(config: &NoMistakesConfig) -> Vec<String> {
    let mut patterns: Vec<_> = config
        .rules
        .iter()
        .filter(|rule| rule.enabled)
        .flat_map(|rule| rule.include.iter().cloned())
        .collect();
    patterns.sort();
    patterns.dedup();
    patterns
}

pub(super) fn include_patterns_by_base_with_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    inferred_roots: &mut no_mistakes::codebase::config::InferredRoots,
) -> BTreeMap<PathBuf, Vec<String>> {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(root);
    let mut patterns = BTreeMap::<PathBuf, Vec<String>>::new();
    for rule in config.rules.iter().filter(|rule| rule.enabled) {
        for include in &rule.include {
            patterns
                .entry(root.clone())
                .or_default()
                .push(include.clone());
            for project in rule
                .projects
                .iter()
                .filter_map(|project_name| config.projects.get(project_name))
            {
                if let Some(project_root) = normalized_project_root(&root, project, inferred_roots)
                {
                    patterns
                        .entry(project_root)
                        .or_default()
                        .push(include.clone());
                }
            }
        }
    }
    for includes in patterns.values_mut() {
        includes.sort();
        includes.dedup();
    }
    patterns
}

pub(super) fn include_preserved_roots(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
) -> Vec<PathBuf> {
    let mut git_files_cache = GitFilesCache::new();
    let mut inferred_roots = no_mistakes::codebase::config::InferredRoots::default();
    collect_preserved_roots(root, config, &mut inferred_roots, |roots, base, include| {
        push_include_preserved_roots(roots, base, include, skip_directories, &mut git_files_cache);
    })
}

pub(super) fn include_preserved_roots_from_files_with_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
    files: &[PathBuf],
    inferred_roots: &mut no_mistakes::codebase::config::InferredRoots,
) -> Vec<PathBuf> {
    collect_preserved_roots(root, config, inferred_roots, |roots, base, include| {
        push_include_preserved_roots_from_files(roots, base, include, skip_directories, files);
    })
}

fn collect_preserved_roots(
    root: &Path,
    config: &NoMistakesConfig,
    inferred_roots: &mut no_mistakes::codebase::config::InferredRoots,
    mut push_include: impl FnMut(&mut Vec<PathBuf>, &Path, &str),
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for rule in config.rules.iter().filter(|rule| rule.enabled) {
        if rule.rule == no_mistakes::codebase::rules::FORBIDDEN_WORKSPACE_CLOSURE {
            for project in rule
                .projects
                .iter()
                .filter_map(|project_name| config.projects.get(project_name))
            {
                if let Some(project_root) = normalized_project_root(root, project, inferred_roots) {
                    roots.push(project_root);
                }
            }
        }
        for include in &rule.include {
            push_include(&mut roots, root, include);
            for project in rule
                .projects
                .iter()
                .filter_map(|project_name| config.projects.get(project_name))
            {
                if let Some(project_root) = normalized_project_root(root, project, inferred_roots) {
                    push_include(&mut roots, &project_root, include);
                }
            }
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

fn normalized_project_root(
    root: &Path,
    project: &no_mistakes::config::v2::schema::Project,
    inferred_roots: &mut no_mistakes::codebase::config::InferredRoots,
) -> Option<PathBuf> {
    super::project_root(root, project, inferred_roots)
        .map(|root| no_mistakes::codebase::ts_resolver::normalize_path(&root))
}

fn push_include_preserved_roots_from_files(
    roots: &mut Vec<PathBuf>,
    base: &Path,
    include: &str,
    skip_directories: &[String],
    files: &[PathBuf],
) {
    if let Some(prefix) = literal_include_prefix(include) {
        roots.push(base.join(&prefix));
    }
    if let Some(suffix) = leading_globstar_literal_prefix(include) {
        let relative_files = files.iter().filter_map(|file| file.strip_prefix(base).ok());
        roots.extend(descendant_dirs_matching_suffix_from_paths(
            base,
            &suffix,
            relative_files,
            skip_directories,
        ));
    }
}

fn push_include_preserved_roots(
    roots: &mut Vec<PathBuf>,
    base: &Path,
    include: &str,
    skip_directories: &[String],
    git_files_cache: &mut GitFilesCache,
) {
    if let Some(prefix) = literal_include_prefix(include) {
        roots.push(base.join(&prefix));
    }
    if let Some(suffix) = leading_globstar_literal_prefix(include) {
        roots.extend(descendant_dirs_matching_suffix(
            base,
            &suffix,
            skip_directories,
            git_files_cache,
        ));
    }
}

#[cfg(test)]
mod tests;
