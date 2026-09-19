impl SharedTraversalContext {
    pub(crate) fn apply_candidate_inventory(&mut self, args: &TraverseArgs) -> Result<()> {
        if self.candidate_inventory_applied || !args.has_candidate_bounds() {
            return Ok(());
        }
        let filtered = self.record_candidate_paths(args)?;
        let subset = self.graph_files.visible_subset(filtered);
        self.escape_universe = Some(std::mem::replace(&mut self.graph_files, subset));
        self.fact_context
            .set_visible_file_set(self.graph_files.visible_path_set());
        self.candidate_inventory_applied = true;
        Ok(())
    }

    pub(crate) fn candidate_graph_files(&self, args: &TraverseArgs) -> Result<graph::GraphFiles> {
        if !args.has_candidate_bounds() {
            return Ok(self
                .graph_files
                .visible_subset(self.graph_files.iter_visible().cloned().collect()));
        }
        let filtered = self.record_candidate_paths(args)?;
        Ok(self.graph_files.visible_subset(filtered))
    }

    pub(crate) fn bounded_lazy_import_graph(
        &self,
        args: &TraverseArgs,
    ) -> Option<&graph::DepGraph> {
        self.bounded_lazy_import_graphs
            .get(&BoundedImportKey::from_args(args))
            .map(std::sync::Arc::as_ref)
    }

    pub(crate) fn seed_bounded_lazy_import_graph_from_args(
        &mut self,
        args: &TraverseArgs,
        cwd: &Path,
    ) -> Result<()> {
        seed_bounded_lazy_import_graph(self, args, cwd)
    }

    fn record_candidate_paths(&self, args: &TraverseArgs) -> Result<Vec<PathBuf>> {
        let inventory = CandidateInventory::new(&args.candidate_include, &args.candidate_exclude)?;
        let visible = self.graph_files.iter_visible().cloned().collect::<Vec<_>>();
        let excluded = visible
            .iter()
            .filter(|path| !inventory.matches(&self.root, path))
            .count();
        let filtered = inventory.filter_paths(&self.root, visible);
        self.session
            .record_work("graph.candidate_files", filtered.len() as u64);
        self.session
            .record_work("graph.candidate_excluded", excluded as u64);
        Ok(filtered)
    }
}

fn bounded_import_only_deps(
    args: &TraverseArgs,
    roots: &[graph::NodeId],
    allowed: Option<&std::collections::HashSet<graph::EdgeKind>>,
    shared: &SharedTraversalContext,
) -> Result<Vec<graph::NodeEntry>> {
    let owned = if shared.candidate_inventory_applied {
        None
    } else {
        Some(shared.candidate_graph_files(args)?)
    };
    let graph_files = owned.as_ref().unwrap_or(&shared.graph_files);
    let sources = shared.dataset.sources_for(&shared.root);
    let workspace = shared.dataset.workspace();
    let overlay = shared.snapshot_resolution_visible(graph_files);
    let (entries, collected) =
        graph::lazy_import_deps_of_with_files_facts_workspace_resolution_cache_and_session(
            graph::LazyImportBuild {
                roots,
                tsconfig: &shared.tsconfig,
                tsconfig_catalog: Some(&shared.tsconfig_catalog),
                max_depth: args.depth,
                graph_files,
                resolution_visible: Some(&overlay),
                allowed,
                facts: graph::LazyImportFacts::new(
                    shared
                        .facts
                        .as_ref()
                        .map(|facts| facts as &dyn graph::TsFactLookup),
                    shared.fact_plan,
                    &shared.fact_context,
                )
                .with_live_cache(&shared.live_lazy_facts)
                .with_source_store(&sources)
                .retain_collected(),
                workspace: &workspace,
                import_resolution_cache: Some(&shared.import_resolution_cache),
            },
            &shared.session,
        );
    shared.publish_lazy_facts(collected);
    Ok(entries)
}

fn escape_reached_files(
    graph_files: &mut graph::GraphFiles,
    collected: &[(PathBuf, crate::codebase::ts_source::facts::TsFileFacts)],
) {
    for (path, _) in collected {
        if !graph_files.contains_visible(path) {
            graph_files.add_explicit_root(path);
        }
    }
}
