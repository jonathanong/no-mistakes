use super::*;

impl<'a> ScopedImportResolver<'a> {
    /// Use only the request's visible universe when resolving files.
    pub fn new(catalog: &'a TsConfigCatalog, visible: &dyn VisiblePathLookup) -> Self {
        Self::build(
            catalog,
            Some(ResolverVisible::Owned(visible.normalized_visible())),
            None,
        )
    }

    pub fn uses_fixed_resolver(&self) -> bool {
        self.fixed_resolver.is_some()
    }

    pub fn scope_key_build_count(&self) -> usize {
        self.scope_key_builds.load(ScopedOrdering::Relaxed)
    }

    pub fn scope_cache_lookup_count(&self) -> usize {
        self.scope_cache_lookups.load(ScopedOrdering::Relaxed)
    }

    pub fn scope_key_count(&self) -> usize {
        self.scope_caches.len()
    }

    #[doc(hidden)]
    pub fn visible_lookup(&self) -> Option<&dyn VisiblePathLookup> {
        self.visible.as_ref().map(ResolverVisible::lookup)
    }
}
