/// Borrowed request-scoped session state for TypeScript import resolution.
pub trait ResolverSession: Send + Sync {
    fn resolver_cache_for_visible(
        &self,
        tsconfig: &TsConfig,
        visible: Option<&dyn VisiblePathLookup>,
    ) -> Arc<ResolverResultCache>;

    fn resolver_cache_for_scope(&self, scope: ResolverCacheScopeKey) -> Arc<ResolverResultCache>;

    fn normalized_visible(&self, visible: &dyn VisiblePathLookup) -> Arc<dyn VisiblePathLookup>;

    fn resolver_observer(&self) -> Option<Arc<dyn crate::hooks::MetricsObserver>>;

    fn path_interner_arc(&self) -> Arc<crate::path_interner::PathInterner>;
}

impl<S: ResolverSession + ?Sized> ResolverSession for Arc<S> {
    fn resolver_cache_for_visible(
        &self,
        tsconfig: &TsConfig,
        visible: Option<&dyn VisiblePathLookup>,
    ) -> Arc<ResolverResultCache> {
        self.as_ref().resolver_cache_for_visible(tsconfig, visible)
    }

    fn resolver_cache_for_scope(&self, scope: ResolverCacheScopeKey) -> Arc<ResolverResultCache> {
        self.as_ref().resolver_cache_for_scope(scope)
    }

    fn normalized_visible(&self, visible: &dyn VisiblePathLookup) -> Arc<dyn VisiblePathLookup> {
        self.as_ref().normalized_visible(visible)
    }

    fn resolver_observer(&self) -> Option<Arc<dyn crate::hooks::MetricsObserver>> {
        self.as_ref().resolver_observer()
    }

    fn path_interner_arc(&self) -> Arc<crate::path_interner::PathInterner> {
        self.as_ref().path_interner_arc()
    }
}

impl<S: ResolverSession + ?Sized> ResolverSession for &S {
    fn resolver_cache_for_visible(
        &self,
        tsconfig: &TsConfig,
        visible: Option<&dyn VisiblePathLookup>,
    ) -> Arc<ResolverResultCache> {
        (*self).resolver_cache_for_visible(tsconfig, visible)
    }

    fn resolver_cache_for_scope(&self, scope: ResolverCacheScopeKey) -> Arc<ResolverResultCache> {
        (*self).resolver_cache_for_scope(scope)
    }

    fn normalized_visible(&self, visible: &dyn VisiblePathLookup) -> Arc<dyn VisiblePathLookup> {
        (*self).normalized_visible(visible)
    }

    fn resolver_observer(&self) -> Option<Arc<dyn crate::hooks::MetricsObserver>> {
        (*self).resolver_observer()
    }

    fn path_interner_arc(&self) -> Arc<crate::path_interner::PathInterner> {
        (*self).path_interner_arc()
    }
}
