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

    /// Return the invocation's canonical, memoized configuration.
    #[doc(hidden)]
    pub fn config(
        &self,
        root: &Path,
        config_path: Option<&Path>,
    ) -> anyhow::Result<Arc<crate::config::v2::NoMistakesConfig>> {
        self.dataset(root).config(config_path)
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
