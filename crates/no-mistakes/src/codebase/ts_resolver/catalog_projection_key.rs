/// Configuration identity for request-owned supplemental resolver projections.
/// Manifest/config contents are frozen by SourceStore for the invocation.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct CatalogProjectionKey {
    forced: bool,
    broken_dirs: Vec<PathBuf>,
    configs: Vec<(PathBuf, ResolverCacheScopeKey)>,
}

impl TsConfigCatalog {
    pub(crate) fn projection_key(&self) -> CatalogProjectionKey {
        CatalogProjectionKey {
            forced: self.forced,
            broken_dirs: self.broken_dirs.clone(),
            configs: self
                .configs
                .iter()
                .map(|config| {
                    (
                        config.path.clone(),
                        ResolverCacheScopeKey::for_lookup(
                            &config.config,
                            None,
                            config.module_resolution.as_deref(),
                            &config.identity,
                        ),
                    )
                })
                .collect(),
        }
    }
}
