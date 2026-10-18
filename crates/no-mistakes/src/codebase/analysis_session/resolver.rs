use super::*;
use crate::codebase::ts_resolver::VisiblePathLookup;

impl AnalysisSession {
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
