impl VisiblePathSnapshot {
    /// Build a request snapshot from candidates already discovered by the
    /// caller. Graph requests use this to share their canonical file set with
    /// specialized collectors instead of starting a second repository scan.
    #[doc(hidden)]
    pub fn from_paths(request_root: &Path, request_paths: &[PathBuf]) -> Self {
        let normalized_request_root = normalize_discovery_path(request_root);
        Self {
            request_root: normalized_request_root,
            authoritative_tracked_paths: false,
            request_view: snapshot_path_view_from_paths(request_paths, None),
            scoped_views: Mutex::new(HashMap::new()),
            observer: None,
        }
    }

    /// Build a snapshot from the caller's authoritative tracked-file inventory.
    /// Every configured scope retains that supplied inventory, including nested
    /// and external Git roots, instead of discovering a competing list.
    #[doc(hidden)]
    pub fn from_tracked_paths(request_root: &Path, request_paths: &[PathBuf]) -> Self {
        let mut snapshot = Self::from_paths(request_root, request_paths);
        snapshot.authoritative_tracked_paths = true;
        Arc::get_mut(&mut snapshot.request_view)
            .expect("new snapshot view is exclusively owned")
            .git_index_available = true;
        snapshot
    }

}
