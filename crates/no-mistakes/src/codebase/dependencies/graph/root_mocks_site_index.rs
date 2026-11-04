type ModuleMemo = FxHashMap<(PathBuf, String), MockedModule>;

struct MockSiteIndex {
    sites: FxHashMap<MockedModule, Vec<usize>>,
    memo: ModuleMemo,
}

struct Collect<'a> {
    facts: &'a dyn TsFactLookup,
    resolver: &'a dyn ImportResolution,
    files: &'a GraphFiles,
    memo: &'a mut ModuleMemo,
    requested: &'a FxHashSet<MockedModule>,
    out: &'a mut FxHashSet<MockedModule>,
}

fn resolve_module(
    resolver: &dyn ImportResolution,
    files: &GraphFiles,
    specifier: &str,
    from: &Path,
) -> MockedModule {
    match resolver.resolve(specifier, from) {
        Some(path) => files
            .visible_path(&path)
            .map(|visible| MockedModule::File(visible.to_path_buf()))
            .unwrap_or_else(|| {
                MockedModule::File(crate::codebase::ts_resolver::normalize_path(&path))
            }),
        None => MockedModule::Specifier(specifier.to_string()),
    }
}

impl DepGraph {
    fn build_mock_site_index(
        &self,
        facts: &dyn TsFactLookup,
        resolver: &dyn ImportResolution,
        files: &GraphFiles,
        requested: &FxHashSet<MockedModule>,
    ) -> MockSiteIndex {
        let (memo, mut pairs) = self
            .resolved_call_sites()
            .par_iter()
            .enumerate()
            .fold(
                || (ModuleMemo::default(), Vec::new()),
                |(mut memo, mut pairs), (index, site)| {
                    let mut out = fx_set();
                    if !site.has_unknown_target() {
                        let mut walk = Collect {
                            facts,
                            resolver,
                            files,
                            memo: &mut memo,
                            requested,
                            out: &mut out,
                        };
                        for candidate in site.targets() {
                            walk.target(site.file.as_path(), candidate.target);
                        }
                    }
                    pairs.extend(out.into_iter().map(|module| (index, module)));
                    (memo, pairs)
                },
            )
            .reduce(
                || (ModuleMemo::default(), Vec::new()),
                |(mut memo, mut pairs), (other_memo, other_pairs)| {
                    memo.extend(other_memo);
                    pairs.extend(other_pairs);
                    (memo, pairs)
                },
            );
        pairs.sort_by_key(|(index, _)| *index);
        let mut sites: FxHashMap<MockedModule, Vec<usize>> = fx_map();
        for (index, module) in pairs {
            sites.entry(module).or_default().push(index);
        }
        MockSiteIndex { sites, memo }
    }
}

impl Collect<'_> {
    fn retain(&mut self, module: MockedModule) {
        if self.requested.contains(&module) {
            self.out.insert(module);
        }
    }

    fn target(&mut self, from: &Path, target: KnownCallTargetRef<'_>) {
        match target {
            KnownCallTargetRef::ModuleExport {
                specifier,
                export_path,
                ..
            } => self.walk(from, specifier, export_path, &mut fx_set()),
            KnownCallTargetRef::RepositoryFunction { file, .. } => {
                self.retain(MockedModule::File(file.to_path_buf()));
            }
            KnownCallTargetRef::Global { .. } => {}
        }
    }

    fn walk(&mut self, from: &Path, specifier: &str, export: &str, seen: &mut ExportSeen) {
        if export.contains('.') {
            return;
        }
        let key = (from.to_path_buf(), specifier.to_string());
        let id = match self.memo.get(&key) {
            Some(id) => id.clone(),
            None => {
                let id = resolve_module(self.resolver, self.files, specifier, from);
                self.memo.insert(key, id.clone());
                id
            }
        };
        self.retain(id.clone());
        let MockedModule::File(path) = id else { return };
        if !seen.insert((path.clone(), export.to_string())) {
            return;
        }
        let hop = {
            let Some(facts) = self.facts.get_ts_facts(&path) else {
                return;
            };
            if facts.parse_error.is_some() || facts.operational_error.is_some() {
                return;
            }
            owned_hop(facts, export)
        };
        match hop {
            Hop::Uncertain | Hop::Miss => {}
            Hop::Next(specifier, name) => self.walk(&path, &specifier, &name, seen),
            Hop::Stars(stars) => {
                for specifier in &stars {
                    self.walk(&path, specifier, export, &mut seen.clone());
                }
            }
        }
    }
}
