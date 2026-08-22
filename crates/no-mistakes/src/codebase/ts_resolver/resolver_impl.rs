impl<'a> ImportResolver<'a> {
    pub fn new(tsconfig: &'a TsConfig) -> Self {
        Self::new_observed(tsconfig, crate::diagnostics::current())
    }

    #[doc(hidden)]
    pub fn new_observed(
        tsconfig: &'a TsConfig,
        observer: Option<std::sync::Arc<crate::diagnostics::InvocationObserver>>,
    ) -> Self {
        Self::from_config(ResolverTsConfig::Borrowed(tsconfig), observer)
    }

    /// Build a persistent resolver that owns its configuration. This is for
    /// request-scoped facades that select a catalog config per importer; the
    /// regular borrowed constructors retain their allocation-free fast path.
    pub(crate) fn new_owned(tsconfig: std::sync::Arc<TsConfig>) -> ImportResolver<'static> {
        ImportResolver::from_config(
            ResolverTsConfig::Owned(tsconfig),
            crate::diagnostics::current(),
        )
    }

    /// Build an owned resolver whose visibility and result cache belong to one
    /// request-scoped analysis session.
    pub(crate) fn new_owned_in_session(
        tsconfig: std::sync::Arc<TsConfig>,
        visible: std::sync::Arc<dyn VisiblePathLookup>,
        session: &crate::codebase::analysis_session::AnalysisSession,
    ) -> ImportResolver<'static> {
        let mut resolver = Self::new_owned(tsconfig);
        let visible_paths = visible.visible_cache_key();
        resolver.cache = session.resolver_cache(resolver.tsconfig(), Some(&visible_paths));
        resolver.visible = Some(ResolverVisible::Owned(visible));
        resolver.session_scoped = true;
        resolver.observer = session.observer().cloned();
        resolver
    }

    fn from_config(
        tsconfig: ResolverTsConfig<'a>,
        observer: Option<std::sync::Arc<crate::diagnostics::InvocationObserver>>,
    ) -> Self {
        let config = tsconfig.get();
        let mut alias_order: Vec<usize> = (0..config.paths.len()).collect();
        alias_order.sort_by(|&a, &b| {
            let la = config.paths[a].0.len();
            let lb = config.paths[b].0.len();
            lb.cmp(&la).then(a.cmp(&b))
        });

        Self {
            tsconfig,
            visible: None,
            alias_order,
            policy: ImportResolutionPolicy::Standard,
            cache_enabled: true,
            cache: std::sync::Arc::new(DashMap::new()),
            shared_cache: None,
            session_scoped: false,
            observer,
        }
    }

    // Preserve the standalone queue analyzer's historical resolution policy.
    pub(crate) fn with_queue_compatibility(mut self, root: &'a Path) -> Self {
        self.cache.clear();
        self.shared_cache = None;
        self.alias_order = (0..self.tsconfig().paths.len()).collect();
        self.policy = ImportResolutionPolicy::QueueCompatibility { root };
        self
    }

    pub(crate) fn new_in_session(
        tsconfig: &'a TsConfig,
        visible: Option<&'a dyn VisiblePathLookup>,
        session: &crate::codebase::analysis_session::AnalysisSession,
    ) -> Self {
        let mut resolver = Self::new_observed(tsconfig, session.observer().cloned());
        let visible_paths = visible.map(VisiblePathLookup::visible_cache_key);
        resolver.cache = session.resolver_cache(tsconfig, visible_paths.as_deref());
        resolver.visible = visible.map(ResolverVisible::Borrowed);
        resolver.session_scoped = true;
        resolver
    }

    pub fn with_visible(self, visible: &'a crate::fx::PathSet) -> Self {
        self.with_visible_lookup(visible)
    }

    pub(crate) fn with_visible_lookup(mut self, visible: &'a dyn VisiblePathLookup) -> Self {
        // Any entries cached before this call were resolved under different
        // visibility (real filesystem, or an earlier `visible` set) and would
        // otherwise leak stale answers into the new scope.
        self.cache.clear();
        self.shared_cache = None;
        self.visible = Some(ResolverVisible::Borrowed(visible));
        self
    }

    /// Keep an owned frozen visibility universe with an owned resolver.
    /// This is intentionally separate from `with_visible` so common borrowed
    /// consumers retain their no-Arc fast path.
    pub(crate) fn with_owned_visible(
        self,
        visible: std::sync::Arc<dyn VisiblePathLookup>,
    ) -> Self {
        self.with_owned_visible_lookup(visible)
    }

    pub(crate) fn with_owned_visible_lookup(
        mut self,
        visible: std::sync::Arc<dyn VisiblePathLookup>,
    ) -> Self {
        self.cache.clear();
        self.shared_cache = None;
        self.visible = Some(ResolverVisible::Owned(visible));
        self
    }

    pub fn without_cache(mut self) -> Self {
        self.cache_enabled = false;
        self
    }

    pub(crate) fn with_shared_cache(mut self, cache: &'a ImportResolutionCache) -> Self {
        self.cache.clear();
        self.shared_cache = Some(cache);
        self
    }

    pub(crate) fn visible_files(&self) -> Option<&dyn VisiblePathLookup> {
        self.visible.as_ref().map(ResolverVisible::lookup)
    }

    /// Returns `true` if `specifier` matches any configured tsconfig path
    /// alias pattern, regardless of whether the target exists on disk. Used by
    /// `resolve-check` to flag a configured alias whose target is missing as a
    /// real error rather than an external/bare specifier.
    pub fn matches_alias(&self, specifier: &str) -> bool {
        self.tsconfig()
            .paths
            .iter()
            .any(|(pattern, _)| match_alias(pattern, specifier).is_some())
    }

    fn tsconfig(&self) -> &TsConfig {
        self.tsconfig.get()
    }
}
