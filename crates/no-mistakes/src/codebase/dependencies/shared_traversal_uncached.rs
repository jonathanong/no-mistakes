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
    let entries = match direction {
        Direction::Deps if import_only => {
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
                        allowed,
                        facts: graph::LazyImportFacts::new(
                            shared
                                .facts
                                .as_ref()
                                .map(|facts| facts as &dyn graph::TsFactLookup),
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
            *shared
                .pending_lazy_facts
                .lock()
                .expect("lazy fact sink is poisoned") = Some(
                crate::codebase::ts_source::facts::TsFactMap::from_iter_with_plan(
                    collected,
                    shared.fact_plan,
                ),
            );
            entries
        }
        Direction::Deps if has_call_relationship(allowed) => {
            let graph = shared.graph_shared()?;
            let call_roots = graph.expand_call_roots(&call_roots(entrypoints));
            let roots = roots_with_call_roots(roots, call_roots, entrypoints, allowed);
            graph.deps_of(&roots, args.depth, allowed)
        }
        Direction::Deps if shared.build_plan.symbols && !args.include_symbols => shared
            .request_graph_without_symbols_shared(allowed)?
            .deps_of(roots, args.depth, allowed),
        Direction::Deps => shared.graph_shared()?.deps_of(roots, args.depth, allowed),
        Direction::Dependents if has_call_relationship(allowed) => {
            let graph = shared.graph_shared()?;
            let call_roots = graph.expand_call_roots(&call_roots(entrypoints));
            let roots = roots_with_call_roots(roots, call_roots, entrypoints, allowed);
            graph.dependents_of(&roots, args.depth, allowed)
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
            graph.dependents_of_symbol_nodes(&roots, args.depth, allowed)
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
            )
        }
        Direction::Dependents if shared.build_plan.symbols && !args.include_symbols => shared
            .request_graph_without_symbols_shared(allowed)?
            .dependents_of(roots, args.depth, allowed),
        Direction::Dependents => shared
            .graph_shared()?
            .dependents_of(roots, args.depth, allowed),
    };
    Ok(entries)
}

fn has_call_relationship(allowed: Option<&std::collections::HashSet<EdgeKind>>) -> bool {
    allowed.is_some_and(|allowed| allowed.contains(&EdgeKind::Call))
}

fn call_roots(entrypoints: &[Entrypoint]) -> Vec<graph::CallRoot> {
    entrypoints
        .iter()
        .filter_map(|entrypoint| {
            let file = entrypoint.node.as_file()?.to_path_buf();
            Some(match &entrypoint.symbol {
                Some(symbol) => graph::CallRoot::Function {
                    file,
                    symbol: symbol.clone(),
                },
                None => graph::CallRoot::File(file),
            })
        })
        .collect()
}

/// Mixed relationship traversals retain file roots for file-level edges and
/// expanded callable roots for call edges. Call-only symbol queries omit the
/// file node so projection does not also walk unrelated top-level calls.
fn roots_with_call_roots(
    roots: &[NodeId],
    call_roots: Vec<NodeId>,
    entrypoints: &[Entrypoint],
    allowed: Option<&std::collections::HashSet<EdgeKind>>,
) -> Vec<NodeId> {
    let call_only = allowed.is_some_and(|kinds| {
        kinds.len() == 1 && kinds.contains(&EdgeKind::Call)
    });
    let mut combined = Vec::with_capacity(roots.len() + call_roots.len());
    combined.extend(
        roots
            .iter()
            .filter(|root| keep_call_projection_file_root(root, entrypoints, call_only))
            .cloned(),
    );
    combined.extend(call_roots);
    combined.sort();
    combined.dedup();
    combined
}

fn keep_call_projection_file_root(
    root: &NodeId,
    entrypoints: &[Entrypoint],
    call_only: bool,
) -> bool {
    if !call_only {
        return true;
    }
    let NodeId::File(file) = root else {
        return true;
    };
    !entrypoints.iter().any(|entrypoint| {
        entrypoint.symbol.is_some() && entrypoint.node.as_file() == Some(file.as_ref())
    })
}
