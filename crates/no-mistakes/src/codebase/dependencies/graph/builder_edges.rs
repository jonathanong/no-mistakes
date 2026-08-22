/// Collects every configured edge kind and merges each into `forward`/`reverse`.
/// Independent core kinds collect `Vec<Edge>` in parallel; merge order stays
/// the historical serial sequence so public graph JSON stays byte-identical.
/// Playwright selector edges still attach after the partial graph in
/// `builder_core.rs`. `graph.ci` still writes maps in place later.
struct EdgeMaps<'a> {
    forward: &'a mut EdgeMap,
    reverse: &'a mut EdgeMap,
    resource_edge_details: &'a mut ResourceEdgeDetails,
    resource_diagnostics: &'a mut Vec<ResourceGraphDiagnostic>,
}

struct EdgeResolutionContext<'a> {
    resolver: &'a dyn ImportResolution,
    session: &'a crate::codebase::analysis_session::AnalysisSession,
}

fn collect_and_merge_all_edges(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    playwright_snapshot: Option<&crate::playwright::fsutil::VisiblePathSnapshot>,
    facts: Option<&dyn TsFactLookup>,
    resolution: EdgeResolutionContext<'_>,
    parsed_imports: &ParsedImports<'_>,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    maps: EdgeMaps<'_>,
) -> Result<()> {
    let EdgeMaps {
        forward,
        reverse,
        resource_edge_details,
        resource_diagnostics,
    } = maps;
    require_core_edge_facts(edge_inputs.plan, facts)?;
    crate::invocation::check_timeout()?;
    let core = collect_independent_core_edges(
        edge_inputs,
        facts,
        resolution.resolver,
        resolution.session,
        parsed_imports,
        workspace,
    );
    merge_independent_core_edges(forward, reverse, core);

    collect_remaining_edges(
        edge_inputs,
        playwright_snapshot,
        facts,
        resolution,
        EdgeMaps {
            forward,
            reverse,
            resource_edge_details,
            resource_diagnostics,
        },
    )
}

fn require_core_edge_facts(plan: GraphBuildPlan, facts: Option<&dyn TsFactLookup>) -> Result<()> {
    if plan.route_imports && facts.is_none() {
        anyhow::bail!("TS import facts are required for route-import edges");
    }
    if plan.symbols && facts.is_none() {
        anyhow::bail!("TS symbol facts are required when symbol edges are requested");
    }
    Ok(())
}

fn collect_import_edges_for_core(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    parsed_imports: &ParsedImports<'_>,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> Vec<Edge> {
    if !edge_inputs.plan.imports {
        return Vec::new();
    }
    collect_import_edges(
        parsed_imports,
        resolver,
        workspace,
        edge_inputs.graph_files,
        &edge_inputs.interner,
    )
}

fn collect_route_import_edges_for_core(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: Option<&dyn TsFactLookup>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Vec<Edge> {
    if !edge_inputs.plan.route_imports {
        return Vec::new();
    }
    collect_route_import_edges(
        edge_inputs.graph_files.indexable(),
        facts.expect("route-import plan requires TS facts"),
        edge_inputs.tsconfig,
        edge_inputs.tsconfig_catalog,
        edge_inputs.graph_files,
        session,
    )
}

fn collect_workspace_edges_for_core(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    parsed_imports: &ParsedImports<'_>,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> Vec<Edge> {
    if !edge_inputs.plan.workspace {
        return Vec::new();
    }
    collect_workspace_edges(
        parsed_imports,
        resolver,
        workspace,
        edge_inputs.graph_files,
        &edge_inputs.interner,
    )
}

fn collect_package_edges_for_core(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> Vec<Edge> {
    if !edge_inputs.plan.package {
        return Vec::new();
    }
    collect_workspace_manifest_edges(
        edge_inputs.graph_files.all(),
        workspace,
        edge_inputs.graph_files,
        &edge_inputs.interner,
    )
}

fn collect_asset_edges_for_core(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    parsed_imports: &ParsedImports<'_>,
    resolver: &dyn ImportResolution,
) -> Vec<Edge> {
    if !edge_inputs.plan.assets {
        return Vec::new();
    }
    collect_asset_edges(
        parsed_imports,
        resolver,
        edge_inputs.graph_files,
        &edge_inputs.interner,
    )
}

fn collect_symbol_edges_for_core(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: Option<&dyn TsFactLookup>,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> Vec<Edge> {
    if !edge_inputs.plan.symbols {
        return Vec::new();
    }
    collect_symbol_edges(
        edge_inputs.root,
        SymbolGraphFiles {
            indexable: edge_inputs.graph_files.indexable(),
            all: edge_inputs.graph_files.all(),
            visible: edge_inputs.graph_files,
            graph_files: edge_inputs.graph_files,
        },
        facts.expect("symbol plan requires TS facts"),
        resolver,
        workspace,
        edge_inputs.config_options,
        &edge_inputs.interner,
    )
}

fn collect_test_edges_for_core(edge_inputs: &GraphEdgeBuildInputs<'_>) -> Vec<Edge> {
    if !edge_inputs.plan.tests {
        return Vec::new();
    }
    collect_test_edges(
        edge_inputs.root,
        edge_inputs.graph_files.indexable(),
        edge_inputs
            .config_options
            .and_then(|options| options.test_filter.as_ref()),
        &edge_inputs.interner,
    )
}
