fn merge_http_process_edges(
    root: &Path,
    tsconfig: &TsConfig,
    plan: GraphBuildPlan,
    facts: Option<&dyn TsFactLookup>,
    graph_files: &GraphFiles,
    config_options: Option<&GraphConfigOptions>,
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
) {
    // HTTP and process collectors consume shared TS facts in this path.
    // Keep the file-content fallback empty so graph builds do not add a
    // second source read pass.
    if plan.http {
        let http_call_edges = collect_http_call_edges(
            root,
            tsconfig,
            facts,
            &[],
            graph_files.indexable(),
            &graph_files.all,
            config_options,
        );
        merge_edges(forward, reverse, http_call_edges);
    }

    if plan.process {
        let spawn_edges = collect_process_spawn_edges(root, facts, &[], graph_files.indexable());
        merge_edges(forward, reverse, spawn_edges);
    }
}

fn merge_swift_edges(
    root: &Path,
    tsconfig: &TsConfig,
    plan: GraphBuildPlan,
    graph_files: &GraphFiles,
    config_options: Option<&GraphConfigOptions>,
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
) {
    if !plan.swift {
        return;
    }

    let swift_edges = collect_swift_edges(root, tsconfig, &graph_files.all, config_options);
    for (from, to, _) in &swift_edges {
        forward.entry(from.clone()).or_default();
        forward.entry(to.clone()).or_default();
    }
    merge_edges(forward, reverse, swift_edges);
}

fn sort_adjacency_lists(forward: &mut EdgeMap, reverse: &mut EdgeMap) {
    // Sort adjacency lists for deterministic BFS output.
    for adj in forward.values_mut().chain(reverse.values_mut()) {
        adj.sort_by_cached_key(|(n, k)| (node_sort_key(n), *k as u8));
        adj.dedup();
    }
}
