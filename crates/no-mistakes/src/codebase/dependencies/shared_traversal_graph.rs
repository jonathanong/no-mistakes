impl SharedTraversalContext {
    pub(crate) fn facts(&mut self) -> &crate::codebase::ts_source::facts::TsFactMap {
        self.ensure_facts();
        self.facts.as_ref().expect("TS facts are initialized")
    }

    fn ensure_facts(&mut self) {
        let remaining = self
            .graph_files
            .indexable()
            .iter()
            .filter(|path| {
                self.facts
                    .as_ref()
                    .is_none_or(|facts| !facts.contains_key(*path))
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

    fn graph(&mut self) -> Result<&graph::DepGraph> {
        if self.graph.is_none() {
            self.graph = Some(self.request_graph(self.build_plan)?);
        }
        self.graph
            .as_deref()
            .context("dependency graph was not initialized")
    }

    fn request_graph_without_symbols(
        &mut self,
        allowed: Option<&std::collections::HashSet<EdgeKind>>,
    ) -> Result<std::sync::Arc<graph::DepGraph>> {
        self.request_graph(graph::GraphBuildPlan::from_allowed(allowed))
    }

    fn request_graph(
        &mut self,
        plan: graph::GraphBuildPlan,
    ) -> Result<std::sync::Arc<graph::DepGraph>> {
        self.ensure_facts();
        let key = EffectiveGraphPlanKey::new(plan, &self.graph_files, self.analysis_generation);
        let builds_before = self.graph_cache.build_count();
        let graph = self.graph_cache.get_or_build(key, || {
            build_canonical_graph(CanonicalGraphBuild {
                root: &self.root,
                tsconfig: &self.tsconfig,
                tsconfig_catalog: &self.tsconfig_catalog,
                plan,
                graph_files: &self.graph_files,
                config_path: self.config_path.as_deref(),
                prepared_graph: &self.prepared_graph,
                facts: self
                    .facts
                    .as_ref()
                    .map(|facts| facts as &dyn graph::TsFactLookup),
                import_resolution_cache: &self.import_resolution_cache,
                dotnet_facts: self
                    .prepared_test_projects
                    .as_ref()
                    .and_then(|projects| projects.dotnet_facts()),
                swift_facts: self
                    .prepared_test_projects
                    .as_ref()
                    .and_then(|projects| projects.swift_facts()),
                visible_paths: self.dataset.visible_paths(),
                session: self.session.clone(),
            })
        });
        self.graph_builds = self.graph_cache.build_count();
        if self.graph_builds == builds_before {
            self.session.record_work("graph.reuses", 1);
        }
        graph
    }

    pub(crate) fn prepare_canonical_graph_with_check_facts(
        &mut self,
        facts: &crate::codebase::check_facts::CheckFactMap,
    ) -> Result<()> {
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
                visible_paths: self.dataset.visible_paths(),
                session: self.session.clone(),
            })
        })?;
        self.graph = Some(graph);
        self.graph_builds = self.graph_cache.build_count();
        Ok(())
    }

    fn symbol_index(&mut self) -> Result<std::sync::Arc<graph::SymbolIndex>> {
        self.ensure_facts();
        let key = GraphFileUniverseKey::new(&self.graph_files, self.analysis_generation);
        let workspace = self.dataset.workspace();
        let builds_before = self.symbol_index_cache.build_count();
        let index = self.symbol_index_cache.get_or_build(key, || {
            Ok(
                graph::SymbolIndex::build_from_facts_workspace_resolution_cache_and_session(
                    &self.tsconfig,
                    Some(&self.tsconfig_catalog),
                    &self.graph_files,
                    self.facts.as_ref().expect("TS facts are initialized"),
                    &workspace,
                    Some(&self.import_resolution_cache),
                    &self.session,
                ),
            )
        });
        self.symbol_index_builds = self.symbol_index_cache.build_count();
        if self.symbol_index_builds == builds_before {
            self.session.record_work("symbol_index.reuses", 1);
        }
        index
    }

    fn invalidate_analysis_caches(&mut self) {
        self.graph = None;
        self.analysis_generation = self.analysis_generation.wrapping_add(1);
        self.graph_cache.clear();
        self.symbol_index_cache.clear();
        self.traversal_results.clear();
    }
}

#[cfg(test)]
mod shared_build_cache_tests;
