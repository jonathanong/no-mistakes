impl<'a> ScopedImportResolver<'a> {
    pub(crate) fn with_queue_compatibility(mut self, root: &'a ScopedPath) -> Self {
        self.fixed_resolver = self.fixed_resolver.take().map(|resolver| {
            let mut resolver = resolver.with_queue_compatibility(root);
            if let Some(session) = self.session {
                let key = ResolverCacheScopeKey::for_lookup(
                    resolver.tsconfig(),
                    resolver.visible_files(),
                    None,
                    &[],
                )
                .with_queue_root(Some(root));
                resolver.cache = session.resolver_cache_for_scope(key);
                resolver.session_scoped = true;
                resolver.session_exact_visibility = resolver.visible.is_some();
            }
            resolver
        });
        self.automatic_fixed_roots = None;
        self.shared_cache = None;
        self.queue_root = Some(root);
        self
    }
}
