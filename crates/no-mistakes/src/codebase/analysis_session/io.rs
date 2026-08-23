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
}
