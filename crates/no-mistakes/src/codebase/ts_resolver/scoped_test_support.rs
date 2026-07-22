use super::*;

impl<'a> ScopedImportResolver<'a> {
    /// Use only the request's visible universe when resolving files.
    pub(crate) fn new(
        catalog: &'a TsConfigCatalog,
        visible: &ScopedHashSet<ScopedPathBuf>,
    ) -> Self {
        Self::build(
            catalog,
            Some(ScopedArc::new(normalized_visible(visible))),
            None,
        )
    }

    pub(crate) fn uses_fixed_resolver(&self) -> bool {
        self.fixed_resolver.is_some()
    }

    pub(crate) fn scope_key_build_count(&self) -> usize {
        self.scope_key_builds.load(ScopedOrdering::Relaxed)
    }

    pub(crate) fn scope_cache_lookup_count(&self) -> usize {
        self.scope_cache_lookups.load(ScopedOrdering::Relaxed)
    }

    pub(crate) fn scope_key_count(&self) -> usize {
        self.scope_caches.len()
    }
}
