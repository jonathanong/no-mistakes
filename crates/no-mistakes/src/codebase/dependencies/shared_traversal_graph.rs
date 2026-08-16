impl SharedTraversalContext {
    pub(crate) fn ensure_facts(&mut self) {
        let remaining = self
            .graph_files
            .indexable()
            .iter()
            .filter(|path| {
                self.facts
                    .as_ref()
                    .is_none_or(|facts| !facts.contains_key(path))
            })
            .cloned()
            .collect::<Vec<_>>();
        if remaining.is_empty() {
            self.facts.get_or_insert_with(|| {
                crate::codebase::ts_source::facts::TsFactMap::from_iter_with_plan(
                    std::iter::empty(),
                    self.fact_plan,
                )
            });
            return;
        }
        let sources = self.dataset.sources_for(&self.root);
        let collected =
            crate::codebase::ts_source::facts::collect_ts_facts_with_context_sources_and_session(
                &self.session,
                &remaining,
                self.fact_plan,
                &self.fact_context,
                &sources,
            );
        self.facts
            .get_or_insert_with(|| {
                crate::codebase::ts_source::facts::TsFactMap::from_iter_with_plan(
                    std::iter::empty(),
                    self.fact_plan,
                )
            })
            .extend(collected);
    }

    pub(crate) fn prepare_canonical_graph_with_check_facts(
        &mut self,
        facts: &crate::codebase::check_facts::CheckFactMap,
    ) -> Result<()> {
        let vitest_setup_projects = self.prepared_vitest_setup_projects();
        let key = EffectiveGraphPlanKey::new(
            self.build_plan,
            &self.graph_files,
            self.analysis_generation,
        );
        let graph = self.graph_cache.get_or_build(key, || {
            build_canonical_graph(CanonicalGraphBuild {
                root: &self.root,
                tsconfig: &self.tsconfig,
                tsconfig_catalog: &self.tsconfig_catalog,
                plan: self.build_plan,
                graph_files: &self.graph_files,
                config_path: self.config_path.as_deref(),
                prepared_graph: &self.prepared_graph,
                facts: Some(facts as &dyn graph::TsFactLookup),
                import_resolution_cache: &self.import_resolution_cache,
                dotnet_facts: self
                    .prepared_test_projects
                    .as_ref()
                    .and_then(|projects| projects.dotnet_facts()),
                swift_facts: self
                    .prepared_test_projects
                    .as_ref()
                    .and_then(|projects| projects.swift_facts()),
                vitest_setup_projects,
                visible_paths: self.dataset.visible_paths(),
                session: self.session.clone(),
            })
        })?;
        self.graph = Some(graph);
        self.graph_builds = self.graph_cache.build_count();
        Ok(())
    }

    fn prepared_vitest_setup_projects(&self) -> Vec<graph::VitestSetupProject> {
        self.prepared_test_projects
            .as_ref()
            .map_or_else(Vec::new, |projects| projects.vitest_setup_projects())
    }

    fn invalidate_analysis_caches(&mut self) {
        self.graph = None;
        self.analysis_generation = self.analysis_generation.wrapping_add(1);
        self.graph_cache.clear();
        self.symbol_index_cache.clear();
        self.traversal_results
            .lock()
            .expect("traversal result cache is poisoned")
            .clear();
        self.graph_builds = self.graph_cache.build_count();
        self.symbol_index_builds = self.symbol_index_cache.build_count();
    }
}

#[cfg(test)]
mod shared_build_cache_tests;
