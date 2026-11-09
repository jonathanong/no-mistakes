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
    shared: &SharedTraversalContext,
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
    let excluded = ordinary_exclusions(shared, entrypoints);
    let entries = match direction {
        Direction::Deps if import_only => import_only_deps(args, roots, allowed, shared)?,
        Direction::Deps if has_callable_relationship(allowed) => {
            let graph = shared.graph_shared()?;
            let call_roots = graph.expand_call_roots(&call_roots(entrypoints));
            let roots = roots_with_call_roots(roots, call_roots, entrypoints, allowed);
            graph.deps_of(&roots, args.depth, allowed)
        }
        Direction::Deps if shared.build_plan.symbols && !args.include_symbols => {
            let graph = shared.request_graph_without_symbols_shared(allowed)?;
            graph.deps_of(roots, args.depth, allowed)
        }
        Direction::Deps => shared.graph_shared()?.deps_of(roots, args.depth, allowed),
        Direction::Dependents if has_callable_relationship(allowed) => {
            let graph = shared.graph_shared()?;
            let call_roots = graph.expand_call_roots(&call_roots(entrypoints));
            let roots = roots_with_call_roots(roots, call_roots, entrypoints, allowed);
            report_dependents(&graph, &roots, args.depth, allowed, &excluded)
        }
        Direction::Dependents if args.include_symbols => {
            let graph = shared.graph_shared()?;
            let roots = roots_with_existing_queue_jobs(
                roots,
                entrypoints,
                graph.as_ref(),
                shared.session.interner(),
            );
            let roots = roots_with_exported_symbol_roots(&roots, graph.as_ref());
            report_symbol_nodes(&graph, &roots, args.depth, allowed, &excluded)
        }
        Direction::Dependents if any_symbol && shared.build_plan.symbols => {
            let graph = shared.request_graph_without_symbols_shared(allowed)?;
            resolve_symbol_dependents(
                &root,
                entrypoints,
                args.depth,
                allowed,
                &graph,
                symbol_index.expect("symbol index is built for symbol dependents"),
                (!excluded.is_empty()).then_some(excluded.as_ref()),
            )
        }
        Direction::Dependents if any_symbol => {
            let graph = shared.graph_shared()?;
            resolve_symbol_dependents(
                &root,
                entrypoints,
                args.depth,
                allowed,
                graph.as_ref(),
                symbol_index.expect("symbol index is built for symbol dependents"),
                (!excluded.is_empty()).then_some(excluded.as_ref()),
            )
        }
        Direction::Dependents if shared.build_plan.symbols && !args.include_symbols => {
            let graph = shared.request_graph_without_symbols_shared(allowed)?;
            report_dependents(&graph, roots, args.depth, allowed, &excluded)
        }
        Direction::Dependents => report_dependents(
            shared.graph_shared()?.as_ref(),
            roots,
            args.depth,
            allowed,
            &excluded,
        ),
    };
    Ok(entries)
}

fn import_only_deps(
    args: &TraverseArgs,
    roots: &[NodeId],
    allowed: Option<&std::collections::HashSet<EdgeKind>>,
    shared: &SharedTraversalContext,
) -> Result<Vec<graph::NodeEntry>> {
    if args.has_candidate_bounds() {
        if let Some(graph) = shared.bounded_lazy_import_graph(args) {
            return Ok(graph.deps_of(roots, args.depth, allowed));
        }
        return bounded_import_only_deps(args, roots, allowed, shared);
    }
    if let Some(graph) = shared.lazy_import_graph() {
        return Ok(graph.deps_of(roots, args.depth, allowed));
    }
    let sources = shared.dataset.sources_for(&shared.root);
    let workspace = shared.dataset.workspace();
    let (entries, collected) =
        graph::lazy_import_deps_of_with_files_facts_workspace_resolution_cache_and_session(
            graph::LazyImportBuild {
                roots,
                tsconfig: &shared.tsconfig,
                tsconfig_catalog: Some(&shared.tsconfig_catalog),
                max_depth: args.depth,
                graph_files: &shared.graph_files,
                resolution_visible: None,
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
