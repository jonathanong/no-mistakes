fn collect_symbol_edges(
    root: &Path,
    graph_files: SymbolGraphFiles<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    config_options: Option<&GraphConfigOptions>,
) -> Vec<Edge> {
    let SymbolGraphFiles {
        indexable: files,
        all: all_files,
        visible: visible_files,
        graph_files,
    } = graph_files;
    let http_route_defs = collect_symbol_http_route_defs(root, all_files, facts, config_options);
    files
        .par_iter()
        .flat_map(|path| {
            collect_symbol_edges_for_file(
                root,
                path,
                facts,
                resolver,
                workspace,
                visible_files,
                graph_files,
                &http_route_defs,
            )
        })
        .collect()
}
