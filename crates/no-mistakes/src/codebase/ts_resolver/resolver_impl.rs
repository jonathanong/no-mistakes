impl<'a> ImportResolver<'a> {
    pub fn new(tsconfig: &'a TsConfig) -> Self {
        Self::new_observed(tsconfig, crate::diagnostics::current())
    }

    #[doc(hidden)]
    pub fn new_observed(
        tsconfig: &'a TsConfig,
        observer: Option<std::sync::Arc<crate::diagnostics::InvocationObserver>>,
    ) -> Self {
        let mut alias_order: Vec<usize> = (0..tsconfig.paths.len()).collect();
        alias_order.sort_by(|&a, &b| {
            let la = tsconfig.paths[a].0.len();
            let lb = tsconfig.paths[b].0.len();
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
        self.alias_order = (0..self.tsconfig.paths.len()).collect();
        self.policy = ImportResolutionPolicy::QueueCompatibility { root };
        self
    }

    pub(crate) fn new_in_session(
        tsconfig: &'a TsConfig,
        visible: Option<&'a HashSet<PathBuf>>,
        session: &crate::codebase::analysis_session::AnalysisSession,
    ) -> Self {
        let mut resolver = Self::new_observed(tsconfig, session.observer().cloned());
        resolver.cache = session.resolver_cache(tsconfig, visible);
        resolver.visible = visible;
        resolver.session_scoped = true;
        resolver
    }

    pub fn with_visible(mut self, visible: &'a HashSet<PathBuf>) -> Self {
        // Any entries cached before this call were resolved under different
        // visibility (real filesystem, or an earlier `visible` set) and would
        // otherwise leak stale answers into the new scope.
        self.cache.clear();
        self.shared_cache = None;
        self.visible = Some(visible);
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

    pub(crate) fn visible_files(&self) -> Option<&HashSet<PathBuf>> {
        self.visible
    }

    /// Returns `true` if `specifier` matches any configured tsconfig path
    /// alias pattern, regardless of whether the target exists on disk. Used by
    /// `resolve-check` to flag a configured alias whose target is missing as a
    /// real error rather than an external/bare specifier.
    pub fn matches_alias(&self, specifier: &str) -> bool {
        self.tsconfig
            .paths
            .iter()
            .any(|(pattern, _)| match_alias(pattern, specifier).is_some())
    }
}
