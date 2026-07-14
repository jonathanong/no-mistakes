struct GraphEdgeBuildInputs<'a> {
    root: &'a Path,
    tsconfig: &'a TsConfig,
    plan: GraphBuildPlan,
    workspace: Option<&'a crate::codebase::workspaces::IndexedWorkspaceMap>,
    graph_files: &'a GraphFiles,
    config_options: Option<&'a GraphConfigOptions>,
    playwright_settings: Option<&'a crate::playwright::config::Settings>,
    config_path: Option<&'a Path>,
    swift_facts: Option<&'a crate::codebase::swift::SwiftFactMap>,
    import_resolution_cache: Option<&'a crate::codebase::ts_resolver::ImportResolutionCache>,
}

fn merge_http_process_edges(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: Option<&dyn TsFactLookup>,
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
) {
    // HTTP and process collectors consume shared TS facts in this path.
    // Keep the file-content fallback empty so graph builds do not add a
    // second source read pass.
    if inputs.plan.http {
        let http_call_edges = collect_http_call_edges(
            inputs.root,
            inputs.tsconfig,
            facts,
            &[],
            inputs.graph_files.indexable(),
            &inputs.graph_files.all,
            inputs.config_options,
        );
        merge_edges(forward, reverse, http_call_edges);
    }

    if inputs.plan.process {
        let spawn_edges = collect_process_spawn_edges(
            inputs.root,
            facts,
            &[],
            inputs.graph_files.indexable(),
            inputs.graph_files.visible(),
        );
        merge_edges(forward, reverse, spawn_edges);
    }
}

fn merge_swift_edges(
    inputs: &GraphEdgeBuildInputs<'_>,
    ts_facts: Option<&dyn TsFactLookup>,
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
) {
    if !inputs.plan.swift {
        return;
    }

    let swift_edges = collect_swift_edges_with_facts(
        inputs.root,
        inputs.tsconfig,
        &inputs.graph_files.all,
        inputs.config_options,
        ts_facts,
        inputs.swift_facts,
    );
    for (from, to, _) in &swift_edges {
        forward.entry(from.clone()).or_default();
        forward.entry(to.clone()).or_default();
    }
    merge_edges(forward, reverse, swift_edges);
}

fn merge_dotnet_edges(
    inputs: &GraphEdgeBuildInputs<'_>,
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
) {
    if !inputs.plan.dotnet {
        return;
    }

    let dotnet_edges =
        collect_dotnet_edges(inputs.root, &inputs.graph_files.all, inputs.config_options);
    for (from, to, _) in &dotnet_edges {
        forward.entry(from.clone()).or_default();
        forward.entry(to.clone()).or_default();
    }
    merge_edges(forward, reverse, dotnet_edges);
}

fn merge_terraform_edges(
    inputs: &GraphEdgeBuildInputs<'_>,
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
) {
    if !inputs.plan.terraform {
        return;
    }

    let terraform_edges =
        collect_terraform_edges(inputs.root, &inputs.graph_files.all, inputs.config_options);
    for (from, to, _) in &terraform_edges {
        forward.entry(from.clone()).or_default();
        forward.entry(to.clone()).or_default();
    }
    merge_edges(forward, reverse, terraform_edges);
}

fn sort_adjacency_lists(forward: &mut EdgeMap, reverse: &mut EdgeMap) {
    // Sort adjacency lists for deterministic BFS output.
    for adj in forward.values_mut().chain(reverse.values_mut()) {
        adj.sort_by_cached_key(|(n, k)| (node_sort_key(n), n.clone(), *k as u8));
        adj.dedup();
    }
}
