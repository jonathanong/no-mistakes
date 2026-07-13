use super::{
    discover_check_files, preserved_roots::include_preserved_roots_from_files,
    unique_exports_project_roots,
};
use no_mistakes::config::v2::NoMistakesConfig;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) struct CheckFileViews {
    pub(crate) filesystem: Vec<PathBuf>,
    pub(crate) graph: Vec<PathBuf>,
}

pub(crate) fn discover_check_file_views(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
    unique_exports_enabled: bool,
) -> CheckFileViews {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(root);
    let root_files = no_mistakes::codebase::ts_source::git_visible_files(&root);
    discover_check_file_views_from_git_files(
        &root,
        config,
        skip_directories,
        unique_exports_enabled,
        root_files,
    )
}

pub(super) fn discover_check_file_views_from_git_files(
    root: &Path,
    config: &NoMistakesConfig,
    skip_directories: &[String],
    unique_exports_enabled: bool,
    root_files: Option<Vec<String>>,
) -> CheckFileViews {
    let Some(root_files) = root_files else {
        return CheckFileViews {
            filesystem: discover_check_files(
                root,
                config,
                skip_directories,
                unique_exports_enabled,
                None,
            ),
            graph: discover_check_files(root, config, &[], unique_exports_enabled, None),
        };
    };

    // Git provides the complete repository universe once. Both consumers are
    // derived from these paths, so preserved includes cannot trigger another
    // repository `git ls-files` invocation.
    let unique_exports_roots = if unique_exports_enabled {
        unique_exports_project_roots(root, config)
    } else {
        Vec::new()
    };
    let mut universe = existing_git_paths(root, root_files);
    for project_root in unique_exports_roots
        .iter()
        .filter(|project_root| !project_root.starts_with(root))
    {
        universe.extend(complete_project_files(project_root));
    }
    universe.sort();
    universe.dedup();

    let graph_preserved_roots = include_preserved_roots_from_files(root, config, &[], &universe);
    let graph = filter_filesystem_view(
        root,
        universe.into_iter(),
        &[],
        &graph_preserved_roots,
        &unique_exports_roots,
    );

    let preserved_roots =
        include_preserved_roots_from_files(root, config, skip_directories, &graph);
    let filesystem = filter_filesystem_view(
        root,
        graph.iter().cloned(),
        skip_directories,
        &preserved_roots,
        &unique_exports_roots,
    );

    CheckFileViews { filesystem, graph }
}

fn complete_project_files(root: &Path) -> Vec<PathBuf> {
    let files = no_mistakes::codebase::ts_source::git_visible_files(root);
    complete_project_files_from_git(root, files)
}

pub(super) fn complete_project_files_from_git(
    root: &Path,
    files: Option<Vec<String>>,
) -> Vec<PathBuf> {
    match files {
        Some(files) => existing_git_paths(root, files),
        None => no_mistakes::codebase::ts_source::walk_files(root, &[]),
    }
}

fn existing_git_paths(root: &Path, files: Vec<String>) -> Vec<PathBuf> {
    files
        .into_iter()
        .map(|relative| no_mistakes::codebase::ts_resolver::normalize_path(&root.join(relative)))
        .filter(|path| path.exists())
        .collect()
}

fn filter_filesystem_view(
    root: &Path,
    files: impl Iterator<Item = PathBuf>,
    skip_directories: &[String],
    preserved_roots: &[PathBuf],
    unique_exports_roots: &[PathBuf],
) -> Vec<PathBuf> {
    let skip_directories = skip_directories.iter().map(String::as_str).collect();
    files
        .filter(|path| {
            allowed_from_root(root, path, &skip_directories, preserved_roots)
                || unique_exports_roots.iter().any(|project_root| {
                    path.starts_with(project_root)
                        && !under_skipped_directory(project_root, path, &skip_directories)
                })
        })
        .collect()
}

fn allowed_from_root(
    root: &Path,
    path: &Path,
    skip_directories: &HashSet<&str>,
    preserved_roots: &[PathBuf],
) -> bool {
    path.starts_with(root)
        && (!under_skipped_directory(root, path, skip_directories)
            || preserved_roots.iter().any(|preserved_root| {
                path.starts_with(preserved_root)
                    && !under_skipped_directory(preserved_root, path, skip_directories)
            }))
}

fn under_skipped_directory(root: &Path, path: &Path, skip_directories: &HashSet<&str>) -> bool {
    path.strip_prefix(root).ok().is_some_and(|relative| {
        relative.components().any(|component| {
            component.as_os_str().to_str().is_some_and(|name| {
                no_mistakes::codebase::ts_source::is_skipped_dir(name)
                    || skip_directories.contains(name)
            })
        })
    })
}
