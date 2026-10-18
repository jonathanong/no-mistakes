impl<'a> ScopedImportResolver<'a> {
    /// Resolve against the real filesystem. This is needed before consumers
    /// remap symlinked route-import paths into their visible graph universe.
    pub(crate) fn unbounded(catalog: &'a TsConfigCatalog) -> Self {
        Self::build(catalog, None, None)
    }

    /// Resolve against a caller-provided frozen visibility view without an
    /// analysis session. Runner-config discovery uses this before the graph
    /// session exists, while still selecting configuration per importer.
    pub(crate) fn from_visible(
        catalog: &'a TsConfigCatalog,
        visible: &dyn VisiblePathLookup,
    ) -> Self {
        Self::build(
            catalog,
            Some(ResolverVisible::Owned(visible.normalized_visible())),
            None,
        )
    }

    /// Reuse the invocation-owned cache registry for each selected config.
    pub(crate) fn new_in_session(
        catalog: &'a TsConfigCatalog,
        visible: &dyn VisiblePathLookup,
        session: &'a crate::codebase::analysis_session::AnalysisSession,
    ) -> Self {
        Self::build(
            catalog,
            Some(ResolverVisible::Owned(session.normalized_visible(visible))),
            Some(session),
        )
    }

    pub(crate) fn from_lookup(
        catalog: &'a TsConfigCatalog,
        visible: &dyn VisiblePathLookup,
        session: Option<&'a crate::codebase::analysis_session::AnalysisSession>,
    ) -> Self {
        Self::build(
            catalog,
            Some(ResolverVisible::Owned(match session {
                Some(session) => session.normalized_visible(visible),
                None => visible.normalized_visible(),
            })),
            session,
        )
    }

    fn build(
        catalog: &'a TsConfigCatalog,
        visible: Option<ResolverVisible<'a>>,
        session: Option<&'a crate::codebase::analysis_session::AnalysisSession>,
    ) -> Self {
        let fixed_resolver = Self::fixed_resolver(catalog, visible.as_ref(), session);
        let automatic_fixed_roots = match (catalog.is_forced(), fixed_resolver.as_ref()) {
            (false, Some(resolver)) => Some((
                canonical_or_normalized(catalog.root_dir()),
                canonical_or_normalized(&resolver.tsconfig().dir),
            )),
            _ => None,
        };
        Self {
            catalog,
            visible,
            fixed_resolver,
            automatic_fixed_roots,
            caches: ScopedDashMap::new(),
            scope_caches: ScopedDashMap::new(),
            importer_selections: ScopedArc::new(ImporterSelectionCache {
                selections: ScopedDashMap::new(),
                builds: ScopedAtomicUsize::new(0),
            }),
            scope_key_builds: ScopedAtomicUsize::new(0),
            scope_cache_lookups: ScopedAtomicUsize::new(0),
            session,
            shared_cache: None,
            queue_root: None,
        }
    }

    fn fixed_resolver(
        catalog: &'a TsConfigCatalog,
        visible: Option<&ResolverVisible<'a>>,
        session: Option<&'a crate::codebase::analysis_session::AnalysisSession>,
    ) -> Option<ImportResolver<'a>> {
        let config = catalog.fixed_config()?;
        let observer = match session {
            Some(session) => session.observer().cloned(),
            None => crate::diagnostics::current(),
        };
        let mut resolver = ImportResolver::new_observed(config, observer);
        resolver.visible = visible.cloned();
        if let Some(session) = session {
            resolver.cache =
                session.resolver_cache_for_visible(config, visible.map(ResolverVisible::lookup));
            resolver.interner = Some(session.interner_arc());
        }
        resolver.session_scoped = session.is_some();
        resolver.session_exact_visibility = session.is_some() && visible.is_some();
        Some(resolver)
    }
}

fn canonical_or_normalized(path: &Path) -> PathBuf {
    match path.canonicalize() {
        Ok(path) => normalize_path(&path),
        Err(_) => normalize_path(path),
    }
}
