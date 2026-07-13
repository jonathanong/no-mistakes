use no_mistakes::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

use matching::{
    descendant_dirs_matching_suffix, descendant_dirs_matching_suffix_from_paths,
    leading_globstar_literal_prefix, literal_include_prefix, GitFilesCache,
};

mod matching;

pub(super) fn include_preserved_roots(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
) -> Vec<PathBuf> {
    let mut git_files_cache = GitFilesCache::new();
    collect_preserved_roots(root, config, |roots, base, include| {
        push_include_preserved_roots(roots, base, include, skip_directories, &mut git_files_cache);
    })
}

pub(super) fn include_preserved_roots_from_files(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
    files: &[PathBuf],
) -> Vec<PathBuf> {
    collect_preserved_roots(root, config, |roots, base, include| {
        push_include_preserved_roots_from_files(roots, base, include, skip_directories, files);
    })
}

fn collect_preserved_roots(
    root: &Path,
    config: &NoMistakesConfig,
    mut push_include: impl FnMut(&mut Vec<PathBuf>, &Path, &str),
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let mut inferred_roots = no_mistakes::codebase::config::InferredRoots::default();
    for rule in config.rules.iter().filter(|rule| rule.enabled) {
        if rule.rule == no_mistakes::codebase::rules::FORBIDDEN_WORKSPACE_CLOSURE {
            for project_name in &rule.projects {
                let Some(project) = config.projects.get(project_name) else {
                    continue;
                };
                if let Some(project_root) = super::project_root(root, project, &mut inferred_roots)
                {
                    roots.push(project_root);
                }
            }
        }
        for include in &rule.include {
            push_include(&mut roots, root, include);
            for project_name in &rule.projects {
                let Some(project) = config.projects.get(project_name) else {
                    continue;
                };
                if let Some(project_root) = super::project_root(root, project, &mut inferred_roots)
                {
                    push_include(&mut roots, &project_root, include);
                }
            }
        }
    }
    roots.sort();
    roots.dedup();
    roots
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
