impl<'a> ImportResolver<'a> {
    pub(crate) fn classify_import(
        &self,
        specifier: &str,
        importing_file: &Path,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &dyn crate::codebase::ts_resolver::VisiblePathLookup,
    ) -> ImportClassification {
        let key = self.classification_key(specifier, importing_file);
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

        // Legacy graph consumers retain full importer keys. Only the prepared
        // resolver cache already owns exact config, policy, and visibility.
        if !self.session_exact_visibility {
            if let Some(cache) = self.shared_cache {
                let key = self.resolve_key(specifier, importing_file);
                return self.resolve_cached(&cache.raw_entries, key, |key| {
                    self.resolve_uncached(&key.specifier, &key.importing_file)
                });
            }
        }
        let key = self.lookup_key(specifier, importing_file);
        self.resolve_cached(&self.cache, key, |key| match key {
            ResolverLookupKey::File(key) => {
                self.resolve_uncached(&key.specifier, &key.importing_file)
            }
            ResolverLookupKey::Directory {
                directory,
                specifier,
            } => self.resolve_from_directory(specifier, Some(directory)),
            ResolverLookupKey::Independent(specifier) => {
                self.resolve_from_directory(specifier, None)
            }
        })
    }

    fn lookup_key(&self, specifier: &str, importing_file: &Path) -> ResolverLookupKey {
        if self.session_exact_visibility {
            if Path::new(specifier).is_absolute() || !self.is_relative_specifier(specifier) {
                return ResolverLookupKey::Independent(self.intern_resolve_specifier(specifier));
            }
            // Normalize the original importer before taking its parent. When
            // the final component is a filename, normalizing its parent is equivalent.
            if let Some(directory) = importing_file
                .parent()
                .filter(|_| importing_file.file_name().is_some())
            {
                return ResolverLookupKey::Directory {
                    directory: self.intern_resolve_path(directory),
                    specifier: self.intern_resolve_specifier(specifier),
                };
            }
        }
        ResolverLookupKey::File(self.resolve_key(specifier, importing_file))
    }

    fn resolve_key(&self, specifier: &str, importing_file: &Path) -> ResolveKey {
        ResolveKey {
            importing_file: self.intern_resolve_path(importing_file),
            specifier: self.intern_resolve_specifier(specifier),
        }
    }

    fn intern_resolve_path(&self, path: &Path) -> Arc<Path> {
        match &self.interner {
            Some(interner) => interner.intern_path(path),
            None => Arc::<Path>::from(normalize_path(path)),
        }
    }

    fn intern_resolve_specifier(&self, specifier: &str) -> Arc<str> {
        match &self.interner {
            Some(interner) => interner.intern_str(specifier),
            None => Arc::from(specifier),
        }
    }

    fn classification_key(&self, specifier: &str, importing_file: &Path) -> ClassificationKey {
        let ResolveKey {
            importing_file,
            specifier,
        } = self.resolve_key(specifier, importing_file);
        let policy = match self.policy {
            ImportResolutionPolicy::Standard => ClassificationPolicy::Standard,
            ImportResolutionPolicy::QueueCompatibility { root } => ClassificationPolicy::Queue {
                root: match &self.interner {
                    Some(interner) => interner.intern_path(root),
                    None => Arc::from(normalize_path(root)),
                },
            },
        };
        ClassificationKey {
            importing_file,
            specifier,
            policy,
        }
    }

    fn resolve_cached<K: Eq + std::hash::Hash>(
        &self,
        cache: &DashMap<K, Option<PathBuf>, rustc_hash::FxBuildHasher>,
        key: K,
        resolve: impl FnOnce(&K) -> Option<PathBuf>,
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
                let resolved = resolve(entry.key());
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
