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
    discover_visible_classified_paths(root)
        .into_iter()
        .map(|entry| entry.path)
        .collect()
}

pub(crate) fn discover_visible_classified_paths(root: &Path) -> Vec<ClassifiedPath> {
    let mut paths: Vec<ClassifiedPath> = match git_ls_paths(root) {
        Some(files) => files
            .into_iter()
            .map(|relative| root.join(relative))
            .filter_map(|path| {
                let metadata = std::fs::symlink_metadata(&path).ok()?;
                Some(ClassifiedPath {
                    classification: FileClassification::from_file_type(
                        &path,
                        metadata.file_type(),
                    ),
                    path,
                })
            })
            .collect(),
        None => WalkBuilder::new(root)
            .hidden(false)
            .require_git(false)
            .build()
            .scan(root.to_path_buf(), |walker_root, entry| {
                Some(entry.ok().and_then(|entry| {
                    if entry.depth() == 0 {
                        *walker_root = entry.path().to_path_buf();
                    }
                    entry.file_type().and_then(|file_type| {
                        (file_type.is_file() || file_type.is_symlink()).then(|| ClassifiedPath {
                            path: rebase_walk_path(root, walker_root, entry.path()),
                            classification: FileClassification::from_file_type(
                                entry.path(),
                                file_type,
                            ),
                        })
                    })
                }))
            })
            .flatten()
            .collect()
    };
    paths.sort_by(|left, right| left.path.cmp(&right.path));
    paths.dedup_by(|left, right| left.path == right.path);
    paths
}

fn rebase_walk_path(request_root: &Path, walker_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(walker_root)
        .map_or_else(|_| path.to_path_buf(), |relative| request_root.join(relative))
}
