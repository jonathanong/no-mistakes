/// Reverse map from `canonicalize(path)` to the discovery spelling.
///
/// Built lazily on the first `visible_path` miss. Discovery identities already
/// live in `visible`; the reverse map exists only for symlink / case-fold
/// lookups. Eagerly filling it is a `realpath` per file and dominates
/// `GraphFiles::from_files` on large monorepos.
struct CanonicalVisible {
    cache: std::sync::Mutex<Option<HashMap<PathBuf, PathBuf>>>,
}

impl CanonicalVisible {
    fn empty() -> Self {
        Self {
            cache: std::sync::Mutex::new(None),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<HashMap<PathBuf, PathBuf>>> {
        self.cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn insert_if_built(&self, canonical: PathBuf, original: PathBuf) {
        if let Some(map) = self.lock().as_mut() {
            map.insert(canonical, original);
        }
    }

    fn get(
        &self,
        all: &[PathBuf],
        visible: &HashSet<PathBuf>,
        canonical: &Path,
    ) -> Option<PathBuf> {
        let mut guard = self.lock();
        if guard.is_none() {
            *guard = Some(build_canonical_visible(all, visible));
        }
        guard.as_ref()?.get(canonical).cloned()
    }
}

fn build_canonical_visible(
    all: &[PathBuf],
    visible: &HashSet<PathBuf>,
) -> HashMap<PathBuf, PathBuf> {
    // `all` is sorted; first discovery spelling wins on a canonical collision.
    let mut map = HashMap::new();
    for path in all.iter().filter(|path| visible.contains(*path)) {
        if let Ok(canonical) = path.canonicalize() {
            map.entry(crate::codebase::ts_resolver::normalize_path(&canonical))
                .or_insert_with(|| path.clone());
        }
    }
    map
}

pub(crate) struct GraphFiles {
    all: Vec<PathBuf>,
    indexable: Vec<PathBuf>,
    visible: HashSet<PathBuf>,
    canonical_visible: CanonicalVisible,
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
        let visible: HashSet<PathBuf> = all.iter().cloned().collect();
        resource_candidates.sort();
        resource_candidates.dedup();
        let indexable = all
            .iter()
            .filter(|path| is_indexable(path) && !excluded_indexable.contains(*path))
            .cloned()
            .collect();
        Self {
            all,
            indexable,
            visible,
            canonical_visible: CanonicalVisible::empty(),
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
                self.canonical_visible.insert_if_built(
                    crate::codebase::ts_resolver::normalize_path(&canonical),
                    path.clone(),
                );
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
        if let Some(path) = self.visible.get(&canonical) {
            return Some(path);
        }
        let original = self
            .canonical_visible
            .get(&self.all, &self.visible, &canonical)?;
        self.visible.get(&original).map(PathBuf::as_path)
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
