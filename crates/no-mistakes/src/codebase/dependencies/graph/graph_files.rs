pub(crate) struct GraphFiles {
    all: Vec<PathBuf>,
    indexable: Vec<PathBuf>,
    visible: HashSet<PathBuf>,
    canonical_visible: HashMap<PathBuf, PathBuf>,
    /// The tracked (or non-Git fallback) files eligible for runtime resource
    /// edges. This intentionally excludes explicit request roots and merely
    /// visible ignored files.
    resource_candidates: Vec<PathBuf>,
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
            snapshot.tracked_paths_from(&all),
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
        let visible: HashSet<PathBuf> = all.iter().cloned().collect();
        resource_candidates.retain(|path| visible.contains(path));
        resource_candidates.sort();
        resource_candidates.dedup();
        let canonical_visible = all
            .iter()
            .filter_map(|path| {
                path.canonicalize().ok().map(|canonical| {
                    (
                        crate::codebase::ts_resolver::normalize_path(&canonical),
                        path.clone(),
                    )
                })
            })
            .collect();
        let indexable = all
            .iter()
            .filter(|path| is_indexable(path) && !excluded_indexable.contains(*path))
            .cloned()
            .collect();
        Self {
            all,
            indexable,
            visible,
            canonical_visible,
            resource_candidates,
        }
    }

    /// Add one existing, explicitly requested file to the request graph.
    ///
    /// This grants authority only to the root target itself. Imports still
    /// resolve against `visible`, so ignored transitive files remain excluded.
    pub(crate) fn add_explicit_root(&mut self, path: &Path) -> bool {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        if !path.is_file() {
            return false;
        }
        let mut changed = false;
        if self.visible.insert(path.clone()) {
            self.all.push(path.clone());
            self.all.sort();
            if let Ok(canonical) = path.canonicalize() {
                self.canonical_visible
                    .insert(crate::codebase::ts_resolver::normalize_path(&canonical), path.clone());
            }
            changed = true;
        }
        // A demand plan may leave an unrequested runner config visible for import resolution
        // while excluding it from eager graph parsing. An explicit query restores that ordinary
        // source file to the indexable universe even though it was already visible.
        if is_indexable(&path) && !self.indexable.contains(&path) {
            self.indexable.push(path);
            self.indexable.sort();
            changed = true;
        }
        changed
    }

    fn is_visible(&self, path: &Path) -> bool {
        self.visible_path(path).is_some()
    }

    pub(crate) fn visible_path(&self, path: &Path) -> Option<&Path> {
        if let Some(path) = self.visible.get(path) {
            return Some(path);
        }
        let canonical = crate::codebase::ts_resolver::normalize_path(&path.canonicalize().ok()?);
        self.canonical_visible.get(&canonical).map(PathBuf::as_path)
    }

    pub(crate) fn indexable(&self) -> &[PathBuf] {
        &self.indexable
    }

    pub(crate) fn all(&self) -> &[PathBuf] {
        &self.all
    }

    pub(crate) fn visible(&self) -> &HashSet<PathBuf> {
        &self.visible
    }

    pub(crate) fn resource_candidates(&self) -> &[PathBuf] {
        &self.resource_candidates
    }
}
