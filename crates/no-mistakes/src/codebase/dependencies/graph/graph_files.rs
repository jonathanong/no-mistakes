pub(crate) struct GraphFiles {
    all: Vec<PathBuf>,
    indexable: Vec<PathBuf>,
    visible: HashSet<PathBuf>,
}

impl GraphFiles {
    pub(crate) fn discover(root: &Path) -> Self {
        Self::from_files(crate::codebase::ts_source::discover_files(root, &[]))
    }

    pub(crate) fn from_files(all: Vec<PathBuf>) -> Self {
        let visible = all.iter().cloned().collect();
        let indexable = all.iter().filter(|p| is_indexable(p)).cloned().collect();
        Self {
            all,
            indexable,
            visible,
        }
    }

    /// Add one existing, explicitly requested file to the request graph.
    ///
    /// This grants authority only to the root target itself. Imports still
    /// resolve against `visible`, so ignored transitive files remain excluded.
    pub(crate) fn add_explicit_root(&mut self, path: &Path) -> bool {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        if !path.is_file() || !self.visible.insert(path.clone()) {
            return false;
        }
        self.all.push(path.clone());
        self.all.sort();
        if is_indexable(&path) {
            self.indexable.push(path);
            self.indexable.sort();
        }
        true
    }

    fn is_visible(&self, path: &Path) -> bool {
        self.visible.contains(path)
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
