struct CatalogBuilder<'a> {
    root: PathBuf,
    root_real: PathBuf,
    visible: BTreeSet<PathBuf>,
    visible_real: BTreeSet<PathBuf>,
    candidate_roots: Vec<PathBuf>,
    sources: Option<&'a crate::codebase::ts_source::SourceStore>,
    // Parsed configs retain lexical paths because they define relative TypeScript semantics.
    states: BTreeMap<PathBuf, Result<EffectiveConfig, String>>,
    // Canonical identities only guard recursive extends cycles across symlink spellings.
    loading: HashSet<PathBuf>,
    diagnostics: BTreeSet<TsConfigDiagnostic>,
    broken_dirs: BTreeSet<PathBuf>,
}

impl<'a> CatalogBuilder<'a> {
    fn new(
        root: &Path,
        candidate_roots: &[PathBuf],
        visible_paths: &[PathBuf],
        sources: Option<&'a crate::codebase::ts_source::SourceStore>,
    ) -> Self {
        let root = normalize_path(root);
        let root_real = real_path(&root).unwrap_or_else(|| root.clone());
        let mut candidate_roots = if candidate_roots.is_empty() {
            vec![root.clone()]
        } else {
            candidate_roots.iter().map(|path| normalize_path(path)).collect()
        };
        if let Ok(workspace) = crate::codebase::workspaces::load_from_files(&root, visible_paths) {
            candidate_roots.extend(workspace.packages.into_iter().map(|package| normalize_path(&package.dir)));
        }
        candidate_roots.sort();
        candidate_roots.dedup();
        Self {
            root,
            root_real,
            visible: visible_paths.iter().map(|path| normalize_path(path)).collect(),
            visible_real: visible_paths.iter().filter_map(|path| real_path(path)).collect(),
            candidate_roots,
            sources,
            states: BTreeMap::new(),
            loading: HashSet::new(),
            diagnostics: BTreeSet::new(),
            broken_dirs: BTreeSet::new(),
        }
    }

    fn build(mut self) -> TsConfigCatalog {
        let mut pending = self.candidates();
        let seeded_configs = self.seeded_configs();
        let mut seen = BTreeSet::new();
        let mut referenced_configs = BTreeSet::new();
        let mut configs = Vec::new();
        while let Some(path) = pending.pop() {
            let path = normalize_path(&path);
            let identity = match real_path(&path) {
                Some(path) => path,
                None => {
                    self.invalid_config(&path, "config file does not exist or cannot be canonicalized".to_string());
                    continue;
                }
            };
            if !seen.insert(identity) {
                continue;
            }
            let dir = match path.parent() {
                Some(parent) => parent.to_path_buf(),
                None => self.root.clone(),
            };
            let effective = match self.load_effective(&path) {
                Ok(effective) => effective,
                Err(error) => {
                    self.invalid_config(&path, error);
                    self.broken_dirs
                        .insert(real_path(&dir).unwrap_or(dir));
                    continue;
                }
            };
            self.queue_references(&path, &effective.references, &mut referenced_configs, &mut pending);
            configs.push(CatalogConfig {
                path: path.clone(),
                config: effective.tsconfig(),
                matcher: effective.matcher(),
                module_resolution: effective.module_resolution.clone(),
                identity: effective.identity.into_iter().collect(),
            });
        }
        configs.sort_by(|left, right| left.path.cmp(&right.path));
        let extended_configs = configs
            .iter()
            .flat_map(|config| config.identity.iter().filter(|identity| *identity != &config.path).cloned())
            .collect::<BTreeSet<_>>();
        configs.retain(|config| {
            !extended_configs.contains(&config.path)
                || seeded_configs.contains(&config.path)
                || referenced_configs.contains(&config.path)
        });
        TsConfigCatalog {
            configs,
            broken_dirs: self.broken_dirs.into_iter().collect(),
            empty: empty_config(&self.root),
            forced: false,
            build_diagnostics: self.diagnostics,
            diagnostics: Mutex::new(BTreeSet::new()),
        }
    }

    fn candidates(&self) -> Vec<PathBuf> {
        let mut paths = BTreeSet::new();
        for root in &self.candidate_roots {
            let root = normalize_path(root);
            let candidate = if root.extension() == Some(std::ffi::OsStr::new("json")) { root } else { root.join("tsconfig.json") };
            if self.visible.contains(&candidate) {
                paths.insert(candidate);
            }
        }
        paths.into_iter().collect()
    }

    // Only direct candidate roots seed ownership. An automatic directory root
    // selects its primary `tsconfig.json`; sibling `tsconfig.*.json` files are
    // auxiliary until a project reference selects them. Explicit callers use
    // the forced catalog to deliberately select a non-primary config.
    fn seeded_configs(&self) -> BTreeSet<PathBuf> {
        self.candidate_roots
            .iter()
            .filter_map(|root| {
                let config = if root.extension() == Some(std::ffi::OsStr::new("json")) {
                    root.clone()
                } else {
                    root.join("tsconfig.json")
                };
                self.is_visible(&config).then(|| normalize_path(&config))
            })
            .collect()
    }

    fn is_visible(&self, path: &Path) -> bool {
        self.visible.contains(path) || self.visible_real.contains(path)
    }
}
