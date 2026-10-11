struct DiscoveredClassifiedPathViews {
    visible: Vec<ClassifiedPath>,
    tracked: Vec<PathBuf>,
    git_index_paths: Vec<PathBuf>,
    metadata_stats: usize,
    git_index_available: bool,
}

fn discover_classified_path_views(root: &Path) -> DiscoveredClassifiedPathViews {
    try_discover_classified_path_views(root).unwrap_or_else(|_| DiscoveredClassifiedPathViews {
        visible: Vec::new(),
        tracked: Vec::new(),
        git_index_paths: Vec::new(),
        metadata_stats: 0,
        git_index_available: false,
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
                git_index_paths: Vec::new(),
                metadata_stats: 0,
                git_index_available: false,
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
        git_index_paths: views.git_index_paths.into_iter().map(|path| root.join(path)).collect(),
        metadata_stats,
        git_index_available: true,
    }
}

fn discover_fallback_classified_paths(root: &Path) -> Vec<ClassifiedPath> {
    let walker_root = std::sync::Arc::new(std::sync::OnceLock::new());
    let paths = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut builder = ignore_walk_builder(root);
    builder
        .hidden(false)
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
        // `ignore` uses its own workers, so cap them at the configured Rayon
        // pool size. This makes CLI `--jobs` the upper bound for this walk.
        .threads(fallback_walk_threads());
    builder.build_parallel().run(|| {
        Box::new(FallbackWalkVisitor {
            walker_root: walker_root.clone(),
            request_root: root.to_path_buf(),
            output: paths.clone(),
            paths: Vec::new(),
        })
    });
    let mut paths = paths
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    paths.sort_by(|left, right| left.path.cmp(&right.path));
    paths.dedup_by(|left, right| left.path == right.path);
    std::mem::take(&mut *paths)
}

pub(super) fn fallback_walk_threads() -> usize {
    rayon::current_num_threads().clamp(1, 12)
}

struct FallbackWalkVisitor {
    walker_root: std::sync::Arc<std::sync::OnceLock<PathBuf>>,
    request_root: PathBuf,
    output: std::sync::Arc<std::sync::Mutex<Vec<ClassifiedPath>>>,
    paths: Vec<ClassifiedPath>,
}

impl ignore::ParallelVisitor for FallbackWalkVisitor {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> ignore::WalkState {
        if crate::invocation::check_timeout().is_err() {
            return ignore::WalkState::Quit;
        }
        let Ok(entry) = entry else {
            // Preserve the serial walk's best-effort handling of entry errors.
            return ignore::WalkState::Continue;
        };
        if entry.depth() == 0 {
            let _ = self.walker_root.set(entry.path().to_path_buf());
        }
        if let Some(file_type) = entry.file_type()
            && (file_type.is_file() || file_type.is_symlink())
        {
            let walk_root = self
                .walker_root
                .get()
                .cloned()
                .unwrap_or_else(|| self.request_root.clone());
            self.paths.push(ClassifiedPath {
                path: rebase_walk_path(&self.request_root, &walk_root, entry.path()),
                classification: FileClassification::from_file_type(entry.path(), file_type),
            });
        }
        ignore::WalkState::Continue
    }
}

impl Drop for FallbackWalkVisitor {
    fn drop(&mut self) {
        self.output
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .append(&mut self.paths);
    }
}
