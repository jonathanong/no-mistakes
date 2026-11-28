use super::*;
use crate::codebase::ts_resolver::VisiblePathLookup;
use crate::codebase::workspace_resolution::IndexedWorkspaceMap;

impl AnalysisSession {
    #[doc(hidden)]
    pub fn workspace(&self, root: &Path) -> Arc<IndexedWorkspaceMap> {
        self.dataset(root).workspace()
    }

    /// Return the invocation-owned result cache for one exact resolver scope.
    pub(crate) fn resolver_cache(
        &self,
        tsconfig: &TsConfig,
        visible: Option<&[std::path::PathBuf]>,
    ) -> Arc<ResolverResultCache> {
        let visible = visible.map(|paths| paths.iter().cloned().collect::<crate::fx::PathSet>());
        self.resolver_cache_for_visible(
            tsconfig,
            visible.as_ref().map(|v| v as &dyn VisiblePathLookup),
        )
    }

    pub(crate) fn resolver_cache_for_scope(
        &self,
        scope: ResolverCacheScopeKey,
    ) -> Arc<ResolverResultCache> {
        match self.resolver_caches.entry(scope) {
            Entry::Occupied(entry) => Arc::clone(entry.get()),
            Entry::Vacant(entry) => {
                self.increment("resolver.scope_constructions", 1);
                let cache = Arc::new(ResolverResultCache::default());
                entry.insert(Arc::clone(&cache));
                cache
            }
        }
    }

    pub(crate) fn resolver_cache_for_visible(
        &self,
        tsconfig: &TsConfig,
        visible: Option<&dyn VisiblePathLookup>,
    ) -> Arc<ResolverResultCache> {
        self.increment("resolver.scope_cache_requests", 1);
        let prepared = visible
            .filter(|visible| visible.scope_identity().is_none())
            .map(|visible| self.prepared_visible(visible));
        self.resolver_cache_for_scope(ResolverCacheScopeKey::for_lookup(
            tsconfig,
            prepared
                .as_deref()
                .map(|v| v as &dyn VisiblePathLookup)
                .or(visible),
            None,
            &[],
        ))
    }
}

impl no_mistakes_ts_analysis::ts_resolver::ResolverSession for AnalysisSession {
    fn resolver_cache_for_visible(
        &self,
        tsconfig: &TsConfig,
        visible: Option<&dyn VisiblePathLookup>,
    ) -> Arc<ResolverResultCache> {
        AnalysisSession::resolver_cache_for_visible(self, tsconfig, visible)
    }

    fn resolver_cache_for_scope(&self, scope: ResolverCacheScopeKey) -> Arc<ResolverResultCache> {
        AnalysisSession::resolver_cache_for_scope(self, scope)
    }

    fn normalized_visible(&self, visible: &dyn VisiblePathLookup) -> Arc<dyn VisiblePathLookup> {
        AnalysisSession::normalized_visible(self, visible)
    }

    fn resolver_observer(
        &self,
    ) -> Option<Arc<dyn no_mistakes_ts_analysis::hooks::MetricsObserver>> {
        self.observer()
            .cloned()
            .map(|observer| observer as Arc<dyn no_mistakes_ts_analysis::hooks::MetricsObserver>)
    }

    fn path_interner_arc(&self) -> Arc<PathInterner> {
        AnalysisSession::interner_arc(self)
    }
}
