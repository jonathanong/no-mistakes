use super::*;

impl AnalysisSession {
    /// Compatibility source gateway backed by the canonical request source
    /// store. New analysis pipelines should retain and pass that store
    /// directly so inventory identity remains explicit.
    pub fn read_source(&self, path: &Path) -> SourceReadResult {
        let path = normalize_path(path);
        self.source_store_for_path(&path)
            .read_path(&path)
            .map_err(|error| SourceReadError {
                path,
                detail: Arc::from(error.to_string()),
            })
    }

    /// Compatibility loader for typed documents that do not have a canonical
    /// dataset cache. Config, tsconfig, workspace, and package callers should
    /// use `AnalysisDataset` or `SourceStore` instead.
    pub fn load_document<T>(
        &self,
        _kind: &'static str,
        _path: &Path,
        load: impl FnOnce() -> anyhow::Result<T>,
    ) -> Result<Arc<T>, DocumentError>
    where
        T: Send + Sync,
    {
        self.increment("manifest.requests", 1);
        self.increment("manifest.parses", 1);
        load().map(Arc::new).map_err(|error| {
            self.increment("manifest.errors", 1);
            DocumentError {
                detail: Arc::from(format!("{error:#}")),
            }
        })
    }

    pub fn parse_document<T>(
        &self,
        kind: &'static str,
        path: &Path,
        parse: impl FnOnce(&str) -> anyhow::Result<T>,
    ) -> Result<Arc<T>, DocumentError>
    where
        T: Send + Sync,
    {
        self.load_document(kind, path, || {
            let source = self
                .read_source(path)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            parse(&source)
        })
    }
}
