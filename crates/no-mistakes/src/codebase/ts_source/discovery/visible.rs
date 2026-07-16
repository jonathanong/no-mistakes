/// Return all tracked and untracked non-ignored files under `root`.
pub fn git_visible_files(root: &Path) -> Option<Vec<String>> {
    git_ls_paths(root).map(|paths| {
        paths
            .into_iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect()
    })
}

/// Return the filesystem entries Git would make visible under `root`.
pub fn discover_visible_paths(root: &Path) -> Vec<PathBuf> {
    try_discover_visible_paths(root).unwrap_or_default()
}

/// Fallible visible-path discovery for invocation boundaries that must preserve
/// Git child-process timeouts rather than returning a partial path set.
#[doc(hidden)]
pub fn try_discover_visible_paths(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    try_discover_visible_classified_paths(root).map(|paths| {
        paths
            .into_iter()
            .map(|entry| entry.path)
            .collect()
    })
}

fn try_discover_visible_classified_paths(root: &Path) -> std::io::Result<Vec<ClassifiedPath>> {
    let paths = try_discover_classified_path_views(root)?.visible;
    if let Err(error) = crate::invocation::check_timeout() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            error.to_string(),
        ));
    }
    Ok(paths)
}

pub(crate) fn discover_visible_classified_paths(root: &Path) -> Vec<ClassifiedPath> {
    try_discover_visible_classified_paths(root).unwrap_or_default()
}

fn rebase_walk_path(request_root: &Path, walker_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(walker_root)
        .map_or_else(|_| path.to_path_buf(), |relative| request_root.join(relative))
}
