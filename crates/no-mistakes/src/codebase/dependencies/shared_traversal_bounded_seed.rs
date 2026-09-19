fn seed_bounded_lazy_import_graph(
    shared: &mut SharedTraversalContext,
    args: &TraverseArgs,
    cwd: &Path,
) -> Result<()> {
    let key = BoundedImportKey::from_args(args);
    if shared.bounded_lazy_import_graphs.contains_key(&key) {
        return Ok(());
    }
    let mut graph_files = if shared.candidate_inventory_applied {
        shared
            .graph_files
            .visible_subset(shared.graph_files.iter_visible().cloned().collect())
    } else {
        shared.candidate_graph_files(args)?
    };
    let explicit = explicit_existing_entry_files(args, &shared.root, cwd);
    for path in &explicit {
        graph_files.add_explicit_root(path);
    }
    let workspace = shared.dataset.workspace();
    let entrypoints = {
        let overlay = shared.snapshot_resolution_visible(&graph_files);
        resolve_entrypoints_with_files_and_workspace(EntrypointResolution {
            raw_entrypoints: &args.files,
            symbol_entrypoints: &args.file_symbols,
            structured_entrypoints: &args.file_entrypoints_are_structured,
            root: &shared.root,
            cwd,
            graph_files: &graph_files,
            visible_lookup: Some(&overlay),
            include_symbols: args.include_symbols,
            workspace: &workspace,
            interner: shared.session.interner(),
        })
    };
    for entrypoint in &entrypoints {
        if let Some(file) = entrypoint.node.as_file() {
            graph_files.add_explicit_root(file);
        }
    }
    let roots: Vec<graph::NodeId> = entrypoints
        .iter()
        .map(|entrypoint| entrypoint.node.clone())
        .collect();
    if roots.is_empty() {
        return Ok(());
    }
    let allowed = relationship_filter(&args.relationships);
    let sources = shared.dataset.sources_for(&shared.root);
    let overlay = shared.snapshot_resolution_visible(&graph_files);
    let ((graph, collected), diagnostics) =
        shared.tsconfig_catalog.isolate_runtime_diagnostics(|| {
            graph::lazy_import_graph_with_session(
                graph::LazyImportBuild {
                    roots: &roots,
                    tsconfig: &shared.tsconfig,
                    tsconfig_catalog: Some(&shared.tsconfig_catalog),
                    max_depth: args.depth,
                    graph_files: &graph_files,
                    resolution_visible: Some(&overlay),
                    allowed: allowed.as_ref(),
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
                &shared.root,
                &shared.session,
            )
        });
    escape_reached_files(&mut graph_files, &collected);
    shared.extend_lazy_facts(
        crate::codebase::ts_source::facts::TsFactMap::from_iter_with_plan(
            collected,
            shared.fact_plan,
        ),
    );
    shared
        .bounded_lazy_import_graphs
        .insert(key.clone(), std::sync::Arc::new(graph));
    shared.bounded_seed_diagnostics.insert(key, diagnostics);
    Ok(())
}
