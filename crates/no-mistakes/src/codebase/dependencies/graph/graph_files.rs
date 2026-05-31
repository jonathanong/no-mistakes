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
        Self { all, indexable, visible }
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
