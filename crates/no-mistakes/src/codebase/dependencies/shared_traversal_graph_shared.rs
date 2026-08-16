impl SharedTraversalContext {
    pub(crate) fn prepared_facts(&self) -> &crate::codebase::ts_source::facts::TsFactMap {
        self.facts.as_ref().expect("TS facts are initialized")
    }

    #[cfg(any(test, feature = "test-instrumentation"))]
    pub(crate) fn graph_build_count(&self) -> usize {
        self.graph_cache.build_count()
    }

    pub(crate) fn graph_shared(&self) -> Result<std::sync::Arc<graph::DepGraph>> {
        if let Some(graph) = &self.graph {
            return Ok(std::sync::Arc::clone(graph));
        }
        self.request_graph_shared(self.build_plan)
    }

    fn request_graph_shared(
        &self,
        plan: graph::GraphBuildPlan,
    ) -> Result<std::sync::Arc<graph::DepGraph>> {
        let vitest_setup_projects = self.prepared_vitest_setup_projects();
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
                facts: Some(self.prepared_facts() as &dyn graph::TsFactLookup),
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
        if self.graph_cache.build_count() == builds_before {
            self.session.record_work("graph.reuses", 1);
        }
        Ok(graph)
    }

    fn request_graph_without_symbols_shared(
        &self,
        allowed: Option<&std::collections::HashSet<EdgeKind>>,
    ) -> Result<std::sync::Arc<graph::DepGraph>> {
        self.request_graph_shared(graph::GraphBuildPlan::from_allowed(allowed))
    }

    fn symbol_index_shared(&self) -> Result<std::sync::Arc<graph::SymbolIndex>> {
        let key = GraphFileUniverseKey::new(&self.graph_files, self.analysis_generation);
        let workspace = self.dataset.workspace();
        let builds_before = self.symbol_index_cache.build_count();
        let index = self.symbol_index_cache.get_or_build(key, || {
            Ok(
                graph::SymbolIndex::build_from_facts_workspace_resolution_cache_and_session(
                    &self.tsconfig,
                    Some(&self.tsconfig_catalog),
                    &self.graph_files,
                    self.prepared_facts(),
                    &workspace,
                    Some(&self.import_resolution_cache),
                    &self.session,
                ),
            )
        })?;
        if self.symbol_index_cache.build_count() == builds_before {
            self.session.record_work("symbol_index.reuses", 1);
        }
        Ok(index)
    }
}
