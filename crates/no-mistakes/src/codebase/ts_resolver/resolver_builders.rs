impl<'a> ImportResolver<'a> {
    pub fn with_visible(self, visible: &'a crate::fx::PathSet) -> Self {
        self.with_visible_lookup(visible)
    }

    pub(crate) fn with_visible_lookup(mut self, visible: &'a dyn VisiblePathLookup) -> Self {
        // Any entries cached before this call were resolved under different
        // visibility (real filesystem, or an earlier `visible` set) and would
        // otherwise leak stale answers into the new scope.
        self.detach_scope_cache();
        self.visible = Some(ResolverVisible::Borrowed(visible));
        self
    }

    /// Keep an owned frozen visibility universe with an owned resolver.
    /// This is intentionally separate from `with_visible` so common borrowed
    /// consumers retain their no-Arc fast path.
    pub(crate) fn with_owned_visible(self, visible: std::sync::Arc<dyn VisiblePathLookup>) -> Self {
        self.with_owned_visible_lookup(visible)
    }

    pub(crate) fn with_owned_visible_lookup(
        mut self,
        visible: std::sync::Arc<dyn VisiblePathLookup>,
    ) -> Self {
        self.detach_scope_cache();
        self.visible = Some(ResolverVisible::Owned(visible));
        self
    }

    pub fn without_cache(mut self) -> Self {
        self.cache_enabled = false;
        self
    }

    pub(crate) fn with_shared_cache(mut self, cache: &'a ImportResolutionCache) -> Self {
        if !self.session_scoped {
            self.cache.clear();
        }
        self.shared_cache = Some(cache);
        self
    }

    pub(crate) fn with_interner(
        mut self,
        interner: Arc<crate::codebase::analysis_session::PathInterner>,
    ) -> Self {
        self.interner = Some(interner);
        self
    }

    pub(crate) fn visible_files(&self) -> Option<&dyn VisiblePathLookup> {
        self.visible.as_ref().map(ResolverVisible::lookup)
    }

    fn detach_scope_cache(&mut self) {
        if self.session_scoped {
            // Other facades still own this exact scope. Changing this facade's
            // semantics must neither clear their entries nor write into them.
            self.cache = Arc::new(ResolverResultCache::default());
        } else {
            self.cache.clear();
        }
        self.shared_cache = None;
        self.session_scoped = false;
        self.session_exact_visibility = false;
    }
}
