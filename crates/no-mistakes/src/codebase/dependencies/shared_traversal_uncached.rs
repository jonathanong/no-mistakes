struct UncachedTraversalRequest<'a> {
    args: &'a TraverseArgs,
    direction: Direction,
    entrypoints: &'a [Entrypoint],
    roots: &'a [NodeId],
    allowed: Option<&'a std::collections::HashSet<EdgeKind>>,
    import_only: bool,
    any_symbol: bool,
    symbol_index: Option<&'a graph::SymbolIndex>,
}

fn collect_uncached_entries(
    request: UncachedTraversalRequest<'_>,
    shared: &mut SharedTraversalContext,
) -> Result<Vec<graph::NodeEntry>> {
    let UncachedTraversalRequest {
        args,
        direction,
        entrypoints,
        roots,
        allowed,
        import_only,
        any_symbol,
        symbol_index,
    } = request;
    let root = shared.root.clone();
    let entries = match direction {
        Direction::Deps if import_only => {
            let sources = shared.dataset.sources_for(&shared.root);
            let workspace = shared.dataset.workspace();
            let (entries, collected) = graph::lazy_import_deps_of_with_files_facts_workspace_resolution_cache_and_session(
                graph::LazyImportBuild {
                    roots,
                    tsconfig: &shared.tsconfig,
                    tsconfig_catalog: Some(&shared.tsconfig_catalog),
                    max_depth: args.depth,
                    graph_files: &shared.graph_files,
                    allowed,
                    facts: graph::LazyImportFacts::new(
                        shared.facts.as_ref().map(|facts| facts as &dyn graph::TsFactLookup),
                        shared.fact_plan,
                        &shared.fact_context,
                    )
                    .with_source_store(&sources)
                    .retain_collected(),
                    workspace: &workspace,
                    import_resolution_cache: Some(&shared.import_resolution_cache),
                },
                &shared.session,
            );
            shared.extend_lazy_facts(
                crate::codebase::ts_source::facts::TsFactMap::from_iter_with_plan(
                    collected,
                    shared.fact_plan,
                ),
            );
            entries
        }
        Direction::Deps if shared.build_plan.symbols && !args.include_symbols => shared
            .request_graph_without_symbols(allowed)?
            .deps_of(roots, args.depth, allowed),
        Direction::Deps => shared.graph()?.deps_of(roots, args.depth, allowed),
        Direction::Dependents if args.include_symbols => {
            let graph = shared.graph()?;
            let roots = roots_with_existing_queue_jobs(roots, entrypoints, graph);
            let roots = roots_with_exported_symbol_roots(&roots, graph);
            graph.dependents_of_symbol_nodes(&roots, args.depth, allowed)
        }
        Direction::Dependents if any_symbol && shared.build_plan.symbols => {
            let graph = shared.request_graph_without_symbols(allowed)?;
            resolve_symbol_dependents(
                &root,
                entrypoints,
                args.depth,
                allowed,
                &graph,
                symbol_index
                    .expect("symbol index is built for symbol dependents"),
            )
        }
        Direction::Dependents if any_symbol => resolve_symbol_dependents(
            &root,
            entrypoints,
            args.depth,
            allowed,
            shared.graph()?,
            symbol_index
                .expect("symbol index is built for symbol dependents"),
        ),
        Direction::Dependents if shared.build_plan.symbols && !args.include_symbols => shared
            .request_graph_without_symbols(allowed)?
            .dependents_of(roots, args.depth, allowed),
        Direction::Dependents => shared
            .graph()?
            .dependents_of(roots, args.depth, allowed),
    };
    Ok(entries)
}
