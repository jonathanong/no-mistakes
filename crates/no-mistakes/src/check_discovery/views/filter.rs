use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn filter_filesystem_view(
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
    (path.starts_with(root) && !under_skipped_directory(root, path, skip_directories))
        || preserved_roots.iter().any(|preserved_root| {
            path.starts_with(preserved_root)
                && !under_skipped_directory(preserved_root, path, skip_directories)
        })
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
