struct GraphEdgeBuildInputs<'a> {
    root: &'a Path,
    tsconfig: &'a TsConfig,
    tsconfig_catalog: Option<&'a crate::codebase::ts_resolver::TsConfigCatalog>,
    plan: GraphBuildPlan,
    workspace: Option<&'a crate::codebase::workspaces::IndexedWorkspaceMap>,
    graph_files: &'a GraphFiles,
    config_options: Option<&'a GraphConfigOptions>,
    playwright_settings: &'a [crate::playwright::config::Settings],
    config_path: Option<&'a Path>,
    dotnet_facts: Option<&'a crate::codebase::dotnet::DotnetFactMap>,
    swift_facts: Option<&'a crate::codebase::swift::SwiftFactMap>,
    import_resolution_cache: Option<&'a crate::codebase::ts_resolver::ImportResolutionCache>,
    visible_paths: Option<&'a crate::codebase::ts_source::VisiblePathSnapshot>,
    workflow_documents: Option<&'a crate::codebase::ci_workflows::ParsedWorkflowSet>,
}

fn parsed_imports_for_plan<'a>(
    plan: GraphBuildPlan,
    files: &'a [PathBuf],
    facts: Option<&'a dyn TsFactLookup>,
) -> Result<ParsedImports<'a>> {
    if !(plan.imports || plan.workspace || plan.assets) {
        return Ok(Vec::new());
    }
    let Some(facts) = facts else {
        anyhow::bail!(
            "TS import facts are required when import, workspace, or asset edges are requested"
        );
    };
    Ok(collect_parsed_imports_from_facts(files, facts))
}

fn collect_http_process_edges(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: Option<&dyn TsFactLookup>,
) -> Vec<Edge> {
    // HTTP and process collectors consume shared TS facts in this path.
    // Keep the file-content fallback empty so graph builds do not add a
    // second source read pass.
    let mut edges = Vec::new();
    if inputs.plan.http {
        edges.extend(collect_http_call_edges(
            inputs.root,
            inputs.tsconfig,
            facts,
            &[],
            inputs.graph_files.indexable(),
            &inputs.graph_files.all,
            inputs.config_options,
        ));
    }
    if inputs.plan.process {
        edges.extend(collect_process_spawn_edges(
            inputs.root,
            facts,
            &[],
            inputs.graph_files.indexable(),
            inputs.graph_files.visible(),
        ));
    }
    edges
}

fn collect_swift_edges_for_plan(
    inputs: &GraphEdgeBuildInputs<'_>,
    ts_facts: Option<&dyn TsFactLookup>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Vec<Edge> {
    if !inputs.plan.swift {
        return Vec::new();
    }
    collect_swift_edges_with_facts(SwiftEdgeInputs {
        root: inputs.root,
        tsconfig: inputs.tsconfig,
        tsconfig_catalog: inputs.tsconfig_catalog,
        all_files: &inputs.graph_files.all,
        config_options: inputs.config_options,
        ts_facts,
        prepared_facts: inputs.swift_facts,
        session,
    })
}

fn collect_dotnet_edges_for_plan(inputs: &GraphEdgeBuildInputs<'_>) -> Vec<Edge> {
    if !inputs.plan.dotnet {
        return Vec::new();
    }
    collect_dotnet_edges(
        inputs.root,
        &inputs.graph_files.all,
        inputs.config_options,
        inputs.dotnet_facts,
    )
}

fn merge_language_frontend_edges(
    inputs: &GraphEdgeBuildInputs<'_>,
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
) {
    let edges = collect_language_frontend_edges(
        inputs.root,
        &inputs.graph_files.all,
        inputs.config_options,
    );
    for (from, to, _) in &edges {
        forward.entry(from.clone()).or_default();
        forward.entry(to.clone()).or_default();
    }
    merge_edges(forward, reverse, edges);
}

fn collect_terraform_edges_for_plan(inputs: &GraphEdgeBuildInputs<'_>) -> Vec<Edge> {
    if !inputs.plan.terraform {
        return Vec::new();
    }
    collect_terraform_edges(inputs.root, &inputs.graph_files.all, inputs.config_options)
}

fn sort_adjacency_lists(forward: &mut EdgeMap, reverse: &mut EdgeMap) {
    // Each map entry is independent, so normalize both directional views in
    // parallel before the final source-ordered flatten assigns ordinals.
    let normalize = |adj: &mut Vec<(NodeId, EdgeKind)>| {
        adj.sort_by(|(left_node, left_kind), (right_node, right_kind)| {
            cmp_node_sort_keys(left_node, right_node)
                .then_with(|| left_node.cmp(right_node))
                .then_with(|| left_kind.sort_key().cmp(&right_kind.sort_key()))
        });
        adj.dedup();
    };
    crate::perf_trace::trace("graph.forward_adjacency_normalization", || {
        forward
            .par_iter_mut()
            .for_each(|(_, adjacent)| normalize(adjacent));
    });
    crate::perf_trace::trace("graph.reverse_adjacency_normalization", || {
        reverse
            .par_iter_mut()
            .for_each(|(_, adjacent)| normalize(adjacent));
    });
}

fn push_route_ref_edge(edges: &mut Vec<Edge>, source: &Path, target: &Path) {
    edges.push((
        NodeId::file(source),
        NodeId::file(target),
        EdgeKind::RouteRef,
    ));
}
