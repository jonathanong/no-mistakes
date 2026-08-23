impl<'a> ImportResolver<'a> {
    pub(crate) fn classify_import(
        &self,
        specifier: &str,
        importing_file: &Path,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &dyn crate::codebase::ts_resolver::VisiblePathLookup,
    ) -> ImportClassification {
        let key = self.resolve_key(specifier, importing_file);
        let classify = || {
            let resolver_target = self.resolve(specifier, importing_file);
            let workspace_target = workspace.resolve_specifier_from_file_visible(
                specifier,
                importing_file,
                visible_files,
            );
            let workspace_recognized = workspace_target.is_some()
                || workspace.recognizes_specifier_from(specifier, importing_file);
            ImportClassification {
                resolver_target,
                workspace_target,
                workspace_recognized,
            }
        };

        let Some(cache) = self.shared_cache else {
            return classify();
        };
        self.increment("resolver.classification_requests", 1);
        cache
            .requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        cache
            .final_entries
            .entry(key)
            .or_insert_with(|| {
                self.increment("resolver.classifications", 1);
                cache
                    .classifications
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                classify()
            })
            .clone()
    }

    pub fn resolve(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf> {
        self.increment("resolver.requests", 1);
        if !self.cache_enabled {
            self.increment("resolver.computations", 1);
            let resolved = self.resolve_uncached(specifier, importing_file);
            self.record_outcome(&resolved);
            return resolved;
        }

        let key = self.resolve_key(specifier, importing_file);

        if let Some(cache) = self.shared_cache {
            return self.resolve_cached(&cache.raw_entries, key, specifier);
        }
        self.resolve_cached(&self.cache, key, specifier)
    }

    fn resolve_key(&self, specifier: &str, importing_file: &Path) -> ResolveKey {
        let importing_file = match &self.interner {
            Some(interner) => interner.intern_path(importing_file),
            None => Arc::<Path>::from(normalize_path(importing_file)),
        };
        let specifier = match &self.interner {
            Some(interner) => interner.intern_str(specifier),
            None => Arc::from(specifier),
        };
        ResolveKey {
            importing_file,
            specifier,
        }
    }

    fn resolve_cached(
        &self,
        cache: &ResolverResultCache,
        key: ResolveKey,
        specifier: &str,
    ) -> Option<PathBuf> {
        if let Some(cached) = cache.get(&key) {
            let resolved = cached.clone();
            drop(cached);
            self.increment("resolver.cache_hits", 1);
            return resolved;
        }
        match cache.entry(key) {
            Entry::Occupied(cached) => {
                let resolved = cached.get().clone();
                drop(cached);
                self.increment("resolver.cache_hits", 1);
                resolved
            }
            Entry::Vacant(entry) => {
                // Hold the shard entry while resolving so concurrent requests
                // cannot repeat successful or failed filesystem work.
                let resolved = self.resolve_uncached(specifier, &entry.key().importing_file);
                drop(entry.insert(resolved.clone()));
                self.increment("resolver.computations", 1);
                if self.session_scoped || self.shared_cache.is_some() {
                    self.increment("resolver.unique_keys", 1);
                }
                self.record_outcome(&resolved);
                resolved
            }
        }
    }

    fn record_outcome(&self, resolved: &Option<PathBuf>) {
        self.increment(
            if resolved.is_some() {
                "resolver.resolved"
            } else {
                "resolver.unresolved"
            },
            1,
        );
    }

    fn increment(&self, metric: &'static str, amount: u64) {
        if let Some(observer) = &self.observer {
            observer.increment(metric, amount);
        }
    }
}
