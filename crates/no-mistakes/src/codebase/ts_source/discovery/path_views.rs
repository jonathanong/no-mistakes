struct DiscoveredClassifiedPathViews {
    visible: Vec<ClassifiedPath>,
    tracked: Vec<PathBuf>,
}

fn git_ls_paths(root: &Path) -> Option<Vec<PathBuf>> {
    git_ls_path_views(root).map(|views| views.visible)
}

fn discover_classified_path_views(root: &Path) -> DiscoveredClassifiedPathViews {
    match git_ls_path_views(root) {
        Some(views) => {
            let tracked_membership = views
                .tracked
                .into_iter()
                .map(|relative| root.join(relative))
                .collect::<HashSet<_>>();
            let visible = classify_existing_paths(root, views.visible);
            let tracked = visible
                .iter()
                .filter(|entry| tracked_membership.contains(&entry.path))
                .map(|entry| entry.path.clone())
                .collect();
            DiscoveredClassifiedPathViews { visible, tracked }
        }
        None => {
            let visible = discover_fallback_classified_paths(root);
            let tracked = visible.iter().map(|entry| entry.path.clone()).collect();
            DiscoveredClassifiedPathViews { visible, tracked }
        }
    }
}

fn classify_existing_paths(root: &Path, paths: Vec<PathBuf>) -> Vec<ClassifiedPath> {
    paths
        .into_iter()
        .filter_map(|relative| {
            let path = root.join(relative);
            let metadata = std::fs::symlink_metadata(&path).ok()?;
            Some(ClassifiedPath {
                classification: FileClassification::from_file_type(&path, metadata.file_type()),
                path,
            })
        })
        .collect()
}

fn discover_fallback_classified_paths(root: &Path) -> Vec<ClassifiedPath> {
    let mut paths = WalkBuilder::new(root)
        .hidden(false)
        .require_git(false)
        .filter_entry(|entry| {
            entry.depth() == 0
                || !entry
                    .file_type()
                    .is_some_and(|file_type| file_type.is_dir())
                || entry.file_name() != ".git"
        })
        .build()
        .scan(root.to_path_buf(), |walker_root, entry| {
            Some(entry.ok().and_then(|entry| {
                if entry.depth() == 0 {
                    *walker_root = entry.path().to_path_buf();
                }
                entry.file_type().and_then(|file_type| {
                    (file_type.is_file() || file_type.is_symlink()).then(|| ClassifiedPath {
                        path: rebase_walk_path(root, walker_root, entry.path()),
                        classification: FileClassification::from_file_type(entry.path(), file_type),
                    })
                })
            }))
        })
        .flatten()
        .collect::<Vec<_>>();
    paths.sort_by(|left, right| left.path.cmp(&right.path));
    paths.dedup_by(|left, right| left.path == right.path);
    paths
}
