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
        resolver.cache =
            session.resolver_cache_for_visible(resolver.tsconfig(), Some(visible.as_ref()));
        resolver.session_exact_visibility = visible.scope_identity().is_some();
        resolver.visible = Some(ResolverVisible::Owned(visible));
        resolver.session_scoped = true;
        resolver.observer = session.observer().cloned();
        resolver.interner = Some(session.interner_arc());
        resolver
    }

    fn from_config(
        tsconfig: ResolverTsConfig<'a>,
        observer: Option<std::sync::Arc<crate::diagnostics::InvocationObserver>>,
    ) -> Self {
        let config = tsconfig.get();
        let mut alias_order: Vec<usize> = (0..config.paths.len()).collect();
        alias_order.sort_by(|&a, &b| {
            let left_star = config.paths[a].0.find('*');
            let right_star = config.paths[b].0.find('*');
            left_star
                .is_some()
                .cmp(&right_star.is_some())
                .then_with(|| {
                    right_star
                        .unwrap_or(config.paths[b].0.len())
                        .cmp(&left_star.unwrap_or(config.paths[a].0.len()))
                })
                .then(a.cmp(&b))
        });

        Self {
            tsconfig,
            visible: None,
            alias_order,
            policy: ImportResolutionPolicy::Standard,
            cache_enabled: true,
            cache: std::sync::Arc::new(DashMap::with_hasher(rustc_hash::FxBuildHasher)),
            shared_cache: None,
            session_scoped: false,
            session_exact_visibility: false,
            observer,
            interner: None,
        }
    }

    // Preserve the standalone queue analyzer's historical resolution policy.
    pub(crate) fn with_queue_compatibility(mut self, root: &'a Path) -> Self {
        self.cache.clear();
        self.shared_cache = None;
        self.session_exact_visibility = false;
        self.alias_order = (0..self.tsconfig().paths.len()).collect();
        self.policy = ImportResolutionPolicy::QueueCompatibility { root };
        self
    }

    pub(crate) fn new_in_session(
        tsconfig: &'a TsConfig,
        visible: Option<&'a dyn VisiblePathLookup>,
        session: &crate::codebase::analysis_session::AnalysisSession,
    ) -> Self {
        let cache = session.resolver_cache_for_visible(tsconfig, visible);
        Self::new_in_session_with_cache(tsconfig, visible, cache, session)
    }

    /// Build a session resolver with a caller-prepared cache for this exact
    /// configuration and visibility universe. Batch callers use this after
    /// resolving their distinct configurations once, avoiding repeated
    /// construction of the repository-sized session cache key per importer.
    pub(crate) fn new_in_session_with_cache(
        tsconfig: &'a TsConfig,
        visible: Option<&'a dyn VisiblePathLookup>,
        cache: std::sync::Arc<ResolverResultCache>,
        session: &crate::codebase::analysis_session::AnalysisSession,
    ) -> Self {
        let mut resolver = Self::new_observed(tsconfig, session.observer().cloned());
        resolver.cache = cache;
        resolver.session_exact_visibility =
            visible.is_some_and(|paths| paths.scope_identity().is_some());
        resolver.visible = visible.map(ResolverVisible::Borrowed);
        resolver.session_scoped = true;
        resolver.interner = Some(session.interner_arc());
        resolver
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

    fn matching_aliases<'resolver>(
        &'resolver self,
        specifier: &'resolver str,
    ) -> impl Iterator<Item = (String, &'resolver [String])> + 'resolver {
        let match_limit = match self.policy {
            ImportResolutionPolicy::Standard => 1,
            ImportResolutionPolicy::QueueCompatibility { .. } => usize::MAX,
        };
        self.alias_order
            .iter()
            .filter_map(move |idx| {
                let (pattern, replacements) = &self.tsconfig().paths[*idx];
                match_alias(pattern, specifier).map(|capture| (capture, replacements.as_slice()))
            })
            .take(match_limit)
    }
}
