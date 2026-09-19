impl SharedTraversalContext {
    pub(crate) fn lazy_import_graph(&self) -> Option<&graph::DepGraph> {
        self.lazy_import_graph.as_deref()
    }

    /// One lazy walk of `args` roots builds the reachable import graph. Later
    /// import-only reports project from that graph instead of walking again.
    pub(crate) fn seed_lazy_import_graph_from_args(
        &mut self,
        args: &TraverseArgs,
        cwd: &Path,
    ) -> Result<()> {
        if self.lazy_import_graph.is_some() {
            return Ok(());
        }
        let explicit = explicit_existing_entry_files(args, &self.root, cwd);
        self.add_explicit_roots(&explicit);
        let workspace = self.dataset.workspace();
        let entrypoints = resolve_entrypoints_with_files_and_workspace(EntrypointResolution {
            raw_entrypoints: &args.files,
            symbol_entrypoints: &args.file_symbols,
            structured_entrypoints: &args.file_entrypoints_are_structured,
            root: &self.root,
            cwd,
            graph_files: &self.graph_files,
            visible_lookup: None,
            include_symbols: args.include_symbols,
            workspace: &workspace,
            interner: self.session.interner(),
        });
        let roots: Vec<graph::NodeId> = entrypoints
            .iter()
            .map(|entrypoint| entrypoint.node.clone())
            .collect();
        if roots.is_empty() {
            return Ok(());
        }
        let allowed = relationship_filter(&args.relationships);
        let sources = self.dataset.sources_for(&self.root);
        let (graph, collected) = graph::lazy_import_graph_with_session(
            graph::LazyImportBuild {
                roots: &roots,
                tsconfig: &self.tsconfig,
                tsconfig_catalog: Some(&self.tsconfig_catalog),
                max_depth: None,
                graph_files: &self.graph_files,
                resolution_visible: None,
                allowed: allowed.as_ref(),
                facts: graph::LazyImportFacts::new(
                    self.facts
                        .as_ref()
                        .map(|facts| facts as &dyn graph::TsFactLookup),
                    self.fact_plan,
                    &self.fact_context,
                )
                .with_live_cache(&self.live_lazy_facts)
                .with_source_store(&sources)
                .retain_collected(),
                workspace: &workspace,
                import_resolution_cache: Some(&self.import_resolution_cache),
            },
            &self.root,
            &self.session,
        );
        self.extend_lazy_facts(
            crate::codebase::ts_source::facts::TsFactMap::from_iter_with_plan(
                collected,
                self.fact_plan,
            ),
        );
        self.lazy_import_graph = Some(std::sync::Arc::new(graph));
        Ok(())
    }
}
