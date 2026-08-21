impl SharedTraversalContext {
    pub(crate) fn use_check_facts(&mut self, shared: &crate::codebase::check_facts::CheckFactMap) {
        self.facts = Some(
            crate::codebase::ts_source::facts::TsFactMap::from_shared_iter_with_plan(
                shared
                    .ts
                    .iter()
                    .map(|(path, facts)| (path.clone(), facts.ts.clone())),
                shared.graph_plan(),
            ),
        );
        self.invalidate_analysis_caches();
    }

    pub(crate) fn seed_cached_program_facts(&mut self, paths: &std::collections::HashSet<PathBuf>) {
        let context = self.fact_context.clone();
        let sources = self.dataset.sources_for(&self.root);
        let session = self.session.clone();
        let collected = crate::codebase::ts_source::facts::TsFactMap::from_iter_with_plan(
            paths
                .iter()
                .filter(|path| {
                    self.facts
                        .as_ref()
                        .is_none_or(|facts| !facts.contains_key(path))
                })
                .filter_map(|path| {
                    let source = sources.read_path(path).ok()?;
                    session
                        .with_recovered_program(path, &source, |program, parsed, error| {
                            error.is_none().then(|| {
                                crate::codebase::ts_source::facts::collect_file_facts_from_program(
                                    path,
                                    self.fact_plan,
                                    &context,
                                    parsed,
                                    program,
                                    None,
                                    self.fact_plan.source.then(|| std::sync::Arc::clone(&source)),
                                )
                            })
                        })
                        .ok()
                        .flatten()
                        .map(|facts| (path.clone(), facts))
                }),
            self.fact_plan,
        );
        self.facts
            .get_or_insert_with(|| {
                crate::codebase::ts_source::facts::TsFactMap::from_iter_with_plan(
                    std::iter::empty(),
                    self.fact_plan,
                )
            })
            .extend(collected);
        self.invalidate_analysis_caches();
    }

    pub(crate) fn extend_lazy_facts(
        &mut self,
        collected: crate::codebase::ts_source::facts::TsFactMap,
    ) {
        if collected.is_empty() {
            return;
        }
        self.facts
            .get_or_insert_with(|| {
                crate::codebase::ts_source::facts::TsFactMap::from_iter_with_plan(
                    std::iter::empty(),
                    self.fact_plan,
                )
            })
            .extend(collected);
        // Keep the traversal we just published. Graph/symbol caches must still
        // rebuild against the merged facts, but the generation stays so the
        // freshly cached traversal remains addressable.
        self.graph = None;
        self.graph_cache.clear();
        self.symbol_index_cache.clear();
        self.graph_builds = self.graph_cache.build_count();
        self.symbol_index_builds = self.symbol_index_cache.build_count();
    }

    pub(crate) fn add_explicit_roots(&mut self, paths: &[PathBuf]) {
        let added = paths
            .iter()
            .filter(|path| self.graph_files.add_explicit_root(path))
            .cloned()
            .collect::<Vec<_>>();
        if added.is_empty() {
            return;
        }
        self.import_resolution_cache.clear();
        self.fact_context
            .set_visible_files(self.graph_files.iter_visible().cloned());
        self.invalidate_analysis_caches();
        // Keep discovery authoritative without eagerly reparsing explicit
        // ignored roots. A prepared supplemental fact view may still supply
        // them later in this request; otherwise `ensure_facts` fills them once
        // immediately before graph construction.
    }
}
