impl TsConfigCatalog {
    /// Build an automatic catalog from the caller's already-discovered paths.
    #[doc(hidden)]
    pub fn from_visible(
        root: &Path,
        candidate_roots: &[PathBuf],
        visible_paths: &[PathBuf],
    ) -> Self {
        CatalogBuilder::new(root, candidate_roots, visible_paths, None).build()
    }

    /// Same as [`Self::from_visible`], reusing the invocation source cache.
    #[doc(hidden)]
    pub fn from_visible_and_sources(
        root: &Path,
        candidate_roots: &[PathBuf],
        visible_paths: &[PathBuf],
        sources: &crate::codebase::ts_source::SourceStore,
    ) -> Self {
        CatalogBuilder::new(root, candidate_roots, visible_paths, Some(sources)).build()
    }

    /// Make the legacy explicit `--tsconfig` behavior available through the
    /// scoped interface. The supplied config deliberately owns every importer.
    #[doc(hidden)]
    pub fn forced(root: &Path, config: TsConfig, path: Option<PathBuf>) -> Self {
        let root = normalize_path(root);
        let path = match path.as_deref().and_then(real_path) {
            Some(path) => path,
            None => normalize_path(&config.dir.join("tsconfig.json")),
        };
        let matcher = ConfigMatcher::all(config.dir.clone());
        Self {
            empty: empty_config(&root),
            configs: vec![CatalogConfig {
                path,
                config,
                matcher,
                module_resolution: None,
                identity: Vec::new(),
            }],
            broken_dirs: Vec::new(),
            forced: true,
            build_diagnostics: BTreeSet::new(),
            diagnostics: Mutex::new(BTreeSet::new()),
        }
    }

    pub(crate) fn config_for(&self, importing_file: &Path) -> &TsConfig {
        self.selection(importing_file)
            .map(|index| &self.configs[index].config)
            .unwrap_or(&self.empty)
    }

    /// Return the one resolver configuration only when it is safe to bypass
    /// per-importer ownership selection. Automatic catalogs remain automatic:
    /// provenance still reports `forced: false`.
    pub(crate) fn fixed_config(&self) -> Option<&TsConfig> {
        let config = self.configs.first()?;
        if self.forced {
            return Some(&config.config);
        }
        if self.configs.len() != 1 || !self.broken_dirs.is_empty() {
            return None;
        }
        let has_diagnostics = !self.build_diagnostics.is_empty()
            || !self
                .diagnostics
                .lock()
                .expect("tsconfig catalog diagnostics mutex poisoned")
                .is_empty();
        if has_diagnostics {
            return None;
        }
        let root_config = real_path(&self.empty.dir.join("tsconfig.json"));
        let root_dir = match real_path(&self.empty.dir) {
            Some(path) => path,
            None => self.empty.dir.clone(),
        };
        (root_config.as_ref() == Some(&config.path) && root_dir == config.matcher.dir)
            .then_some(&config.config)
    }

    pub(crate) fn provenance_for(&self, importing_file: &Path) -> TsConfigProvenance {
        let importer = normalize_path(importing_file);
        let config = self
            .selection(&importer)
            .map(|index| self.configs[index].path.clone());
        TsConfigProvenance {
            importer,
            config,
            forced: self.forced,
        }
    }

    pub(crate) fn resolver_scope_for(
        &self,
        importing_file: &Path,
    ) -> (&TsConfig, Option<&str>, &[PathBuf]) {
        self.resolver_scope_at(self.selection(importing_file))
    }

    /// Select a catalog entry once for a persistent consumer. The index is
    /// request-local and is only meaningful with this catalog instance.
    pub(crate) fn resolver_scope_index_for(&self, importing_file: &Path) -> Option<usize> {
        self.selection(importing_file)
    }

    pub(crate) fn resolver_scope_at(
        &self,
        index: Option<usize>,
    ) -> (&TsConfig, Option<&str>, &[PathBuf]) {
        index.map_or((&self.empty, None, &[]), |index| {
            let config = &self.configs[index];
            (
                &config.config,
                config.module_resolution.as_deref(),
                &config.identity,
            )
        })
    }

    pub(crate) fn diagnostics(&self) -> Vec<TsConfigDiagnostic> {
        let mut diagnostics = self.build_diagnostics.iter().cloned().collect::<Vec<_>>();
        diagnostics.extend(self.runtime_diagnostics());
        diagnostics
    }

    pub(crate) fn runtime_diagnostics(&self) -> Vec<TsConfigDiagnostic> {
        self.diagnostics
            .lock()
            .expect("tsconfig catalog diagnostics mutex poisoned")
            .iter()
            .cloned()
            .collect()
    }

    pub(crate) fn clear_runtime_diagnostics(&self) {
        self.diagnostics
            .lock()
            .expect("tsconfig catalog diagnostics mutex poisoned")
            .clear();
    }

    pub(crate) fn replay_runtime_diagnostics(&self, diagnostics: &[TsConfigDiagnostic]) {
        self.diagnostics
            .lock()
            .expect("tsconfig catalog diagnostics mutex poisoned")
            .extend(diagnostics.iter().cloned());
    }

    pub(crate) fn is_forced(&self) -> bool {
        self.forced
    }

    pub(crate) fn root_dir(&self) -> &Path {
        &self.empty.dir
    }
}
