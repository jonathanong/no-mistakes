enum SnapshotPathKind {
    Visible,
    Tracked,
    GitIndex,
}

struct ProjectedSnapshotPaths {
    visible: Arc<Vec<PathBuf>>,
    tracked: Arc<Vec<PathBuf>>,
    git_index: Arc<Vec<PathBuf>>,
}

impl VisiblePathSnapshot {
    /// Complete prepared Git index file membership, including sparse or locally
    /// deleted entries. These path names do not imply readable source text.
    #[doc(hidden)]
    pub fn git_index_paths_for(&self, root: &Path) -> Arc<Vec<PathBuf>> {
        let view = self.path_view_for(root);
        self.project_paths(root, &view, SnapshotPathKind::GitIndex)
    }

    fn project_paths(
        &self,
        root: &Path,
        view: &Arc<SnapshotPathView>,
        kind: SnapshotPathKind,
    ) -> Arc<Vec<PathBuf>> {
        // Request-view reuse and explicit supplied inventories retain their
        // canonical universe; additional shared Git views stay project-scoped.
        if Arc::ptr_eq(view, &self.request_view) {
            return match kind {
                SnapshotPathKind::Visible => view.sources.inventory().paths(),
                SnapshotPathKind::Tracked => Arc::clone(&view.tracked_paths),
                SnapshotPathKind::GitIndex => Arc::clone(&view.git_index_paths),
            };
        }
        let root = normalize_discovery_path(root);
        let projection = {
            let mut projections = self
                .projected_paths
                .lock()
                .expect("snapshot projection mutex poisoned");
            Arc::clone(projections.entry(root.clone()).or_default())
        };
        // Concurrent callers share one filtering pass and immutable arrays for
        // this project, while retaining the boundary's frozen source inventory.
        let paths = projection.get_or_init(|| {
            increment(&self.observer, "discovery.projections", 1);
            let filter = |paths: &[PathBuf]| {
                Arc::new(paths.iter().filter(|path| path.starts_with(&root)).cloned().collect())
            };
            ProjectedSnapshotPaths {
                visible: filter(&view.sources.inventory().paths()),
                tracked: filter(&view.tracked_paths),
                git_index: filter(&view.git_index_paths),
            }
        });
        match kind {
            SnapshotPathKind::Visible => Arc::clone(&paths.visible),
            SnapshotPathKind::Tracked => Arc::clone(&paths.tracked),
            SnapshotPathKind::GitIndex => Arc::clone(&paths.git_index),
        }
    }
}

fn normalized_index_paths(paths: &[PathBuf]) -> Arc<Vec<PathBuf>> {
    let mut paths = paths.iter().map(|path| normalize_discovery_path(path)).collect::<Vec<_>>();
    sort_os_str_paths(&mut paths);
    paths.dedup();
    Arc::new(paths)
}
