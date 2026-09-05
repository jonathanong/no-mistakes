use super::*;

impl AnalysisSession {
    /// Read through the canonical store for the most-specific prepared dataset.
    /// This keeps source identity stable when one fact universe spans roots.
    pub fn read_source(&self, path: &Path) -> SourceReadResult {
        let path = normalize_path(path);
        self.source_store_for_path(&path)
            .read_path(&path)
            .map_err(|error| SourceReadError {
                path,
                detail: Arc::from(error.to_string()),
            })
    }

    /// Return the prepared dataset source store for `root` when discovery has
    /// already run. Missing roots stay `None` so callers can keep a file-list
    /// inventory instead of discovering the whole tree.
    pub(crate) fn existing_sources_for(
        &self,
        root: &Path,
    ) -> Option<Arc<crate::codebase::ts_source::SourceStore>> {
        if root.as_os_str().is_empty() {
            return None;
        }
        let root = normalize_path(root);
        self.datasets
            .get(&root)
            .and_then(|cell| cell.get().cloned())
            .map(|dataset| dataset.sources_for(&root))
    }

    /// Return the invocation's canonical, memoized configuration.
    #[doc(hidden)]
    pub fn config(
        &self,
        root: &Path,
        config_path: Option<&Path>,
    ) -> anyhow::Result<Arc<crate::config::v2::NoMistakesConfig>> {
        self.dataset(root).config(config_path)
    }

    /// Return the invocation's canonical configuration and selected source path.
    #[doc(hidden)]
    pub fn config_with_path(
        &self,
        root: &Path,
        config_path: Option<&Path>,
    ) -> anyhow::Result<(Arc<crate::config::v2::NoMistakesConfig>, Option<PathBuf>)> {
        self.dataset(root).config_with_path(config_path)
    }

    /// Return the invocation's canonical, memoized TypeScript configuration.
    #[doc(hidden)]
    pub fn tsconfig(
        &self,
        root: &Path,
        tsconfig_path: Option<&Path>,
    ) -> anyhow::Result<Arc<crate::codebase::ts_resolver::TsConfig>> {
        self.dataset(root).tsconfig(tsconfig_path)
    }

    /// Request-scoped filter. Seed with [`Self::insert_test_file_filter`] to skip glob compile.
    #[doc(hidden)]
    pub fn test_file_filter(
        &self,
        root: &Path,
        config: &crate::config::v2::NoMistakesConfig,
    ) -> Arc<crate::codebase::test_filter::TestFileFilter> {
        self.test_file_filter_with_visible(root, config, None)
    }

    pub(crate) fn test_file_filter_with_visible(
        &self,
        root: &Path,
        config: &crate::config::v2::NoMistakesConfig,
        visible_paths: Option<&[PathBuf]>,
    ) -> Arc<crate::codebase::test_filter::TestFileFilter> {
        let root = normalize_path(root);
        let cell = self.test_filter_cell(&root);
        Arc::clone(cell.get_or_init(|| {
            self.increment("test_filter.builds", 1);
            let paths = visible_paths
                .map(<[PathBuf]>::to_vec)
                .unwrap_or_else(|| self.visible_paths(&root).paths_for(&root).as_ref().clone());
            Arc::new(crate::codebase::test_filter::TestFileFilter::from_visible(
                &root, config, &paths,
            ))
        }))
    }

    /// Seed a filter built from prepared project globs. First writer wins.
    #[doc(hidden)]
    pub fn insert_test_file_filter(
        &self,
        root: &Path,
        filter: crate::codebase::test_filter::TestFileFilter,
    ) {
        let cell = self.test_filter_cell(&normalize_path(root));
        let _ = cell.get_or_init(|| Arc::new(filter));
    }

    fn test_filter_cell(&self, root: &Path) -> Arc<super::TestFilterCell> {
        match self.test_filters.entry(root.to_path_buf()) {
            Entry::Occupied(entry) => Arc::clone(entry.get()),
            Entry::Vacant(entry) => {
                let cell = Arc::new(OnceLock::new());
                entry.insert(Arc::clone(&cell));
                cell
            }
        }
    }
}
