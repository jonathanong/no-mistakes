impl VisiblePathSnapshot {
    /// Complete prepared Git index file membership, including sparse or locally
    /// deleted entries. These path names do not imply readable source text.
    #[doc(hidden)]
    pub fn git_index_paths_for(&self, root: &Path) -> Arc<Vec<PathBuf>> {
        let view = self.path_view_for(root);
        self.project_paths(root, &view, Arc::clone(&view.git_index_paths))
    }

    fn project_paths(&self, root: &Path, view: &Arc<SnapshotPathView>, paths: Arc<Vec<PathBuf>>) -> Arc<Vec<PathBuf>> {
        // Request-view reuse and explicit supplied inventories retain their
        // canonical universe; additional shared Git views stay project-scoped.
        if Arc::ptr_eq(view, &self.request_view) {
            return paths;
        }
        let root = normalize_discovery_path(root);
        Arc::new(paths.iter().filter(|path| path.starts_with(&root)).cloned().collect())
    }
}

fn normalized_index_paths(paths: &[PathBuf]) -> Arc<Vec<PathBuf>> {
    let mut paths = paths.iter().map(|path| normalize_discovery_path(path)).collect::<Vec<_>>();
    sort_os_str_paths(&mut paths);
    paths.dedup();
    Arc::new(paths)
}
