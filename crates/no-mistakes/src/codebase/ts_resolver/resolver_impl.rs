impl<'a> ImportResolver<'a> {
    pub fn new(tsconfig: &'a TsConfig) -> Self {
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
            cache: DashMap::new(),
            shared_cache: None,
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

    pub(crate) fn classify_import(
        &self,
        specifier: &str,
        importing_file: &Path,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &HashSet<PathBuf>,
    ) -> ImportClassification {
        let key = ResolveKey {
            importing_file: normalize_path(importing_file),
            specifier: specifier.to_string(),
        };
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
        cache
            .requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        cache
            .final_entries
            .entry(key)
            .or_insert_with(|| {
                cache
                    .classifications
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                classify()
            })
            .clone()
    }

    pub fn resolve(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf> {
        if !self.cache_enabled {
            return self.resolve_uncached(specifier, importing_file);
        }

        let importing_file = normalize_path(importing_file);
        let key = ResolveKey {
            importing_file: importing_file.clone(),
            specifier: specifier.to_string(),
        };

        if let Some(cache) = self.shared_cache {
            return cache
                .raw_entries
                .entry(key)
                .or_insert_with(|| self.resolve_uncached(specifier, importing_file.as_path()))
                .clone();
        }

        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }
        let resolved = self.resolve_uncached(specifier, importing_file.as_path());
        self.cache.insert(key, resolved.clone());
        resolved
    }

    fn resolve_uncached(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf> {
        let is_relative = match self.policy {
            ImportResolutionPolicy::Standard => specifier.starts_with("./") || specifier.starts_with("../"),
            ImportResolutionPolicy::QueueCompatibility { .. } => specifier.starts_with('.'),
        };
        if is_relative {
            let dir = importing_file.parent()?;
            return self.try_path(&dir.join(specifier));
        }

        for idx in &self.alias_order {
            let (pattern, replacements) = &self.tsconfig.paths[*idx];
            if let Some(capture) = match_alias(pattern, specifier) {
                for replacement in replacements {
                    let resolved = replacement.replace('*', &capture);
                    let base = self
                        .tsconfig
                        .base_url
                        .as_ref()
                        .unwrap_or(&self.tsconfig.paths_dir)
                        .join(&resolved);
                    if let Some(p) = self.try_path(&base) {
                        return Some(p);
                    }
                }
            }
        }

        if let Some(base_url) = &self.tsconfig.base_url {
            if let Some(p) = self.try_path(&base_url.join(specifier)) {
                return Some(p);
            }
        }

        if let ImportResolutionPolicy::QueueCompatibility { root } = self.policy {
            return self.try_path(&root.join(specifier));
        }

        None
    }

    /// Try `base` as-is, then with each known extension appended, then as an index file.
    fn try_path(&self, base: &Path) -> Option<PathBuf> {
        if matches!(self.policy, ImportResolutionPolicy::QueueCompatibility { .. }) {
            return self.try_queue_compatibility_path(base);
        }
        let base = normalize_path(base);
        if has_explicit_extension(&base) {
            // NodeNext/ESM: for an emitted `.js`/`.mjs`/`.cjs` specifier:
            // 1. TypeScript source (`.ts`/`.tsx`/`.mts`/`.cts`) — highest priority.
            // 2. Literal file — takes precedence over `.jsx` and declarations.
            // 3. `.jsx`/`.d.*` fallbacks — only when the literal file is absent.
            if let Some(source) = self.try_emitted_source(&base) {
                return Some(source);
            }
            if self.path_is_file(&base) {
                return Some(base);
            }
            return self.try_emitted_fallback(&base);
        }
        if self.path_is_file(&base) {
            return Some(base);
        }
        let s = base.to_string_lossy();

        for ext in EXTENSIONS {
            let candidate = PathBuf::from(format!("{}{}", s, ext));
            if self.path_exists(&candidate) {
                return Some(candidate);
            }
        }

        for ext in EXTENSIONS {
            let candidate = base.join(format!("index{}", ext));
            if self.path_exists(&candidate) {
                return Some(candidate);
            }
        }

        None
    }

}
