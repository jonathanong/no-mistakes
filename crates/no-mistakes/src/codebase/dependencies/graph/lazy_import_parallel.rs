fn with_import_graph_pool<T: Send>(run: impl FnOnce() -> T + Send) -> T {
    // N-API `compute` runs on a libuv worker. The process-global Rayon pool
    // is initialized from that thread and does not keep cores busy. `check`
    // initializes Rayon on the CLI main thread and parses Filaments in ~21s.
    rayon::ThreadPoolBuilder::new()
        .num_threads(
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(8),
        )
        .build()
        .expect("import-graph Rayon pool")
        .install(run)
}

/// Build the reachable import graph the way `check` keeps cores busy: one
/// `par_iter` fact collection over the indexable universe, then one `par_iter`
/// of import-edge emission from those facts. Wave-synchronous lazy BFS left
/// workers idle; `no-mistakes check` parses the Filaments corpus in ~20s.
fn lazy_import_walk_parallel(
    input: LazyImportBuild<'_>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> LazyImportWalk {
    let LazyImportBuild {
        roots,
        tsconfig,
        tsconfig_catalog,
        graph_files,
        allowed,
        facts,
        workspace,
        import_resolution_cache,
        ..
    } = input;
    let files = graph_files.indexable();
    let fact_plan = facts.collect_plan;
    with_import_graph_pool(|| {
        let collected = match facts.prepared {
            Some(prepared)
                if files
                    .iter()
                    .all(|path| prepared.get_ts_facts(path).is_some()) =>
            {
                None
            }
            _ => Some(crate::ast::with_owned_request_parse_cache(|| {
                match facts.sources {
                Some(sources) => crate::codebase::ts_source::facts::collect_ts_facts_with_context_sources_and_session(
                    session,
                    files,
                    fact_plan,
                    facts.context,
                    sources,
                ),
                None => crate::codebase::ts_source::facts::collect_ts_facts_with_session_and_context(
                    session,
                    files,
                    fact_plan,
                    facts.context,
                ),
            }
            })),
        };
        let fact_lookup: &dyn TsFactLookup = match (&collected, facts.prepared) {
            (Some(map), _) => map,
            (None, Some(prepared)) => prepared,
            (None, None) => unreachable!("import-graph seed always has facts or collects them"),
        };
        let resolver = crate::codebase::ts_resolver::ProjectImportResolver::new(
            tsconfig,
            tsconfig_catalog,
            graph_files,
            import_resolution_cache,
            session,
        );
        let interner = session.interner();
        let root_nodes: FxHashSet<NodeId> = roots.iter().cloned().collect();
        let expansions: Vec<(NodeId, Vec<(NodeId, EdgeKind)>)> = files
            .par_iter()
            .map(|path| {
                crate::invocation::check_timeout().ok().map(|()| {
                    let from = NodeId::file_in(interner, path);
                    let neighbors = fact_lookup
                        .get_ts_facts(path)
                        .map(|file_facts| {
                            import_neighbors_from_facts(
                                path,
                                file_facts,
                                &resolver,
                                workspace,
                                graph_files,
                                allowed,
                                interner,
                            )
                        })
                        .unwrap_or_default();
                    (from, neighbors)
                })
            })
            .while_some()
            .collect();
        let mut edges = Vec::new();
        let mut nodes: FxHashSet<NodeId> = root_nodes.clone();
        for (from, neighbors) in expansions {
            nodes.insert(from.clone());
            for (neighbor, kind) in neighbors {
                if is_symbol_owner_bridge(&from, &neighbor) && !root_nodes.contains(&from) {
                    continue;
                }
                nodes.insert(neighbor.clone());
                edges.push(CanonicalEdge::new(from.clone(), neighbor, kind));
            }
        }
        session.record_work("traversal.lazy_nodes", nodes.len() as u64);
        session.record_work("traversal.lazy_parallel_expand", 1);
        LazyImportWalk {
            entries: Vec::new(),
            facts: collected
                .map(|map| map.into_iter().collect())
                .unwrap_or_default(),
            edges,
            nodes: nodes.into_iter().collect(),
        }
    })
}
