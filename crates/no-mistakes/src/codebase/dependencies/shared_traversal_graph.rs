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
        let collected = crate::codebase::ts_source::facts::collect_ts_facts_with_context(
            &remaining,
            self.fact_plan,
            &self.fact_context,
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
            self.ensure_facts();
            let graph = graph::DepGraph::build_with_plan_files_prepared_config_and_facts(
                &self.root,
                &self.tsconfig,
                self.build_plan,
                &self.graph_files,
                self.config_path.as_deref(),
                &self.prepared_graph,
                self.facts
                    .as_ref()
                    .map(|facts| facts as &dyn graph::TsFactLookup),
            )?;
            self.graph = Some(graph);
            self.graph_builds += 1;
        }
        self.graph
            .as_ref()
            .context("dependency graph was not initialized")
    }

    fn request_graph_without_symbols(
        &mut self,
        allowed: Option<&std::collections::HashSet<EdgeKind>>,
    ) -> Result<graph::DepGraph> {
        self.ensure_facts();
        graph::DepGraph::build_with_plan_files_prepared_config_and_facts(
            &self.root,
            &self.tsconfig,
            graph::GraphBuildPlan::from_allowed(allowed),
            &self.graph_files,
            self.config_path.as_deref(),
            &self.prepared_graph,
            self.facts
                .as_ref()
                .map(|facts| facts as &dyn graph::TsFactLookup),
        )
    }
}
