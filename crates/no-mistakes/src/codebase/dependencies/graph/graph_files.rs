pub(crate) struct GraphFiles {
    all: std::sync::Arc<Vec<PathBuf>>,
    indexable: std::sync::Arc<Vec<PathBuf>>,
    /// 1 if `all[i]` is visible. Kept parallel to `all` so lookup can binary
    /// search paths without cloning them into a second set.
    visible: Vec<u8>,
    canonical_visible: CanonicalVisible,
    /// The tracked (or non-Git fallback) files eligible for runtime resource
    /// edges. This intentionally excludes explicit request roots and merely
    /// visible ignored files.
    resource_candidates: std::sync::Arc<Vec<PathBuf>>,
}

impl GraphFiles {
    pub(crate) fn discover(root: &Path) -> Self {
        // Keep the visible and tracked inventories from one discovery. In a
        // Git worktree visible untracked files may participate in import
        // resolution, but must not become implicit runtime-resource targets.
        let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
        let all = crate::codebase::ts_source::discover_files_from_visible(
            root,
            &[],
            &snapshot.paths_for(root),
        );
        Self::from_files_with_resource_candidates(
            all.clone(),
            // Resource candidates are deliberately derived before source
            // discovery filters `fixtures`, `dist`, and similar directories.
            // They are runtime inputs, not files to parse or resolve imports
            // from, and remain subject to resource-target safety checks.
            snapshot.tracked_paths_for(root).as_ref().clone(),
        )
    }

    pub(crate) fn from_files(all: Vec<PathBuf>) -> Self {
        let resource_candidates = all.clone();
        Self::from_files_with_resource_candidates_excluding_indexable(
            all,
            resource_candidates,
            &HashSet::new(),
        )
    }

    /// Construct a graph universe with an explicit tracked-resource subset.
    /// Callers that already hold a `VisiblePathSnapshot` must use this rather
    /// than treating every visible path as tracked.
    pub(crate) fn from_files_with_resource_candidates(
        all: Vec<PathBuf>,
        resource_candidates: Vec<PathBuf>,
    ) -> Self {
        Self::from_files_with_resource_candidates_excluding_indexable(
            all,
            resource_candidates,
            &HashSet::new(),
        )
    }

    pub(crate) fn from_files_with_resource_candidates_excluding_indexable(
        mut all: Vec<PathBuf>,
        mut resource_candidates: Vec<PathBuf>,
        excluded_indexable: &HashSet<PathBuf>,
    ) -> Self {
        all.sort();
        all.dedup();
        let visible = vec![1u8; all.len()];
        resource_candidates.sort();
        resource_candidates.dedup();
        let indexable: Vec<PathBuf> = all
            .iter()
            .filter(|path| is_indexable(path) && !excluded_indexable.contains(*path))
            .cloned()
            .collect();
        let all = std::sync::Arc::new(all);
        let resource_candidates = if resource_candidates.as_slice() == all.as_slice() {
            std::sync::Arc::clone(&all)
        } else {
            std::sync::Arc::new(resource_candidates)
        };
        Self {
            all,
            indexable: std::sync::Arc::new(indexable),
            visible,
            canonical_visible: CanonicalVisible::empty(),
            resource_candidates,
        }
    }

    pub(crate) fn universe_identity(&self) -> &std::sync::Arc<()> {
        self.canonical_visible.universe()
    }

    pub(crate) fn indexable(&self) -> &[PathBuf] {
        &self.indexable
    }

    pub(crate) fn all(&self) -> &[PathBuf] {
        &self.all
    }

    pub(crate) fn resource_candidates(&self) -> &[PathBuf] {
        &self.resource_candidates
    }
}
