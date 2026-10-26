fn collect_symbol_edges_for_core(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: Option<&dyn TsFactLookup>,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> Vec<Edge> {
    let mut selectors = if edge_inputs.plan.symbol_import_selectors {
        collect_symbol_import_selectors(
            edge_inputs,
            facts.expect("symbol selector plan requires facts"),
            resolver,
            workspace,
        )
    } else {
        Vec::new()
    };
    if !edge_inputs.plan.symbols {
        return selectors;
    }
    selectors.extend(collect_symbol_edges(
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
    ));
    selectors
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
