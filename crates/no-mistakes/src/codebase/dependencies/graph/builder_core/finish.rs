{
        crate::invocation::check_timeout()?;
        let call_interner = edge_inputs.interner.clone();
        let mut callable_nodes = if plan.calls {
            files
                .iter()
                .flat_map(|path| {
                    facts
                        .and_then(|facts| facts.get_ts_facts(path))
                        .into_iter()
                        .flat_map({
                            let interner = call_interner.clone();
                            move |file| {
                                let interner = interner.clone();
                                file.callable_scopes.iter().flat_map(move |scope| {
                                    let matches = file
                                        .callable_scope_ids
                                        .iter()
                                        .filter(|(_, candidate)| candidate == scope)
                                        .map(|(id, _)| {
                                            NodeId::callable_in(&interner, path, scope, *id)
                                        })
                                        .collect::<Vec<_>>();
                                    if matches.is_empty() {
                                        vec![NodeId::symbol_in(&interner, path, scope)]
                                    } else {
                                        matches
                                    }
                                })
                            }
                        })
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        callable_nodes.sort();
        callable_nodes.dedup();
        let mut callable_nodes_by_file: FxHashMap<PathBuf, Vec<NodeId>> = fx_map();
        for node in &callable_nodes {
            if let NodeId::Symbol { file, .. } = node {
                callable_nodes_by_file
                    .entry(file.as_ref().to_path_buf())
                    .or_default()
                    .push(node.clone());
            }
        }
        resolved_call_sites.sort_by(|left: &ResolvedCallSite, right| {
            (
                &left.file,
                left.line,
                left.offset,
                &left.caller,
                &left.source_callee,
            )
                .cmp(&(
                &right.file,
                right.line,
                right.offset,
                &right.caller,
                &right.source_callee,
            ))
        });
        let mut graph = Self {
            root: root.to_path_buf(),
            edges: edge_index_from_maps(forward, reverse),
            callable_nodes_by_file,
            resolved_call_sites,
            vitest_setup_projects: Vec::new(),
            effective_edges: OnceLock::new(),
            parse_errors,
            resource_edge_details,
            resource_diagnostics,
        };
        if plan.playwright_selectors {
            let snapshot = playwright_snapshot
                .as_ref()
                .expect("Playwright selector plan prepares a visible-path snapshot");
            let selector_edges = crate::perf_trace::trace("graph.playwright_selectors", || {
                collect_playwright_selector_edges_with_graph(
                    root,
                    config_path,
                    PlaywrightSelectorEdgeInputs {
                        all_files: graph_files.all(),
                        facts,
                        partial_graph: plan.route_imports.then_some(&graph),
                        graph_tsconfig: plan.route_imports.then_some(tsconfig),
                        snapshot,
                        prepared_settings: edge_inputs.playwright_settings,
                        interner: &edge_inputs.interner,
                    },
                )
            })?;
            // Route-import selector analysis reads the partial graph, so keep
            // its historical reconstruction path. Direct selectors can append
            // to the finished index without renumbering the base graph.
            crate::perf_trace::trace("graph.playwright_selector_merge", || {
                graph.append_canonical_edges(selector_edges);
            });
        }
        record_graph_observability(&graph, &session);
        Ok(graph)
}
