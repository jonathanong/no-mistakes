struct DiscoveredClassifiedPathViews {
    visible: Vec<ClassifiedPath>,
    tracked: Vec<PathBuf>,
    metadata_stats: usize,
}

fn discover_classified_path_views(root: &Path) -> DiscoveredClassifiedPathViews {
    try_discover_classified_path_views(root).unwrap_or_else(|_| DiscoveredClassifiedPathViews {
        visible: Vec::new(),
        tracked: Vec::new(),
        metadata_stats: 0,
    })
}

fn try_discover_classified_path_views(
    root: &Path,
) -> std::io::Result<DiscoveredClassifiedPathViews> {
    Ok(match git_ls_path_views(root)? {
        Some(views) => classify_git_path_views(root, views),
        None => {
            let visible = discover_fallback_classified_paths(root);
            let tracked = visible.iter().map(|entry| entry.path.clone()).collect();
            DiscoveredClassifiedPathViews {
                visible,
                tracked,
                metadata_stats: 0,
            }
        }
    })
}

fn classify_git_path_views(root: &Path, views: DiscoveredPathViews) -> DiscoveredClassifiedPathViews {
    let tracked_membership = views
        .tracked
        .into_iter()
        .map(|relative| root.join(relative))
        .collect::<HashSet<_>>();
    let (visible, metadata_stats) = file_inventory::classify_git_listed_paths(
        root,
        views
            .visible
            .into_iter()
            .map(|entry| (entry.path, entry.index_kind))
            .collect(),
    );
    let tracked = visible
        .iter()
        .filter(|entry| tracked_membership.contains(&entry.path))
        .map(|entry| entry.path.clone())
        .collect();
    DiscoveredClassifiedPathViews {
        visible,
        tracked,
        metadata_stats,
    }
}

fn discover_fallback_classified_paths(root: &Path) -> Vec<ClassifiedPath> {
    let mut paths = WalkBuilder::new(root)
        .hidden(false)
        .require_git(false)
        .filter_entry(|entry| {
            if crate::invocation::check_timeout().is_err() {
                return false;
            }
            entry.depth() == 0
                || !entry
                    .file_type()
                    .is_some_and(|file_type| file_type.is_dir())
                || entry.file_name() != ".git"
        })
        .build()
        .take_while(|_| crate::invocation::check_timeout().is_ok())
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
