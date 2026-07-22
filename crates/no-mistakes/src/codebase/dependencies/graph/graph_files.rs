pub(crate) struct GraphFiles {
    all: Vec<PathBuf>,
    indexable: Vec<PathBuf>,
    visible: HashSet<PathBuf>,
    canonical_visible: HashMap<PathBuf, PathBuf>,
}

impl GraphFiles {
    pub(crate) fn discover(root: &Path) -> Self {
        Self::from_files(crate::codebase::ts_source::discover_files(root, &[]))
    }

    pub(crate) fn from_files(all: Vec<PathBuf>) -> Self {
        Self::from_files_excluding_indexable(all, &HashSet::new())
    }

    pub(crate) fn from_files_excluding_indexable(
        all: Vec<PathBuf>,
        excluded_indexable: &HashSet<PathBuf>,
    ) -> Self {
        let visible = all.iter().cloned().collect();
        let canonical_visible = all
            .iter()
            .filter_map(|path| {
                path.canonicalize()
                    .ok()
                    .map(|canonical| (crate::codebase::ts_resolver::normalize_path(&canonical), path.clone()))
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

}
