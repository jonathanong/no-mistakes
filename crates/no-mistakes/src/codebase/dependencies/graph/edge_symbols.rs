fn collect_symbol_edges(
    root: &Path,
    graph_files: SymbolGraphFiles<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    config_options: Option<&GraphConfigOptions>,
    interner: &PathInterner,
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
            collect_symbol_edges_for_file(SymbolFileEdgeInputs {
                root,
                path,
                facts,
                resolver,
                workspace,
                visible_files,
                graph_files,
                http_route_defs: &http_route_defs,
                interner,
            })
        })
        .collect()
}

struct SymbolFileEdgeInputs<'a> {
    root: &'a Path,
    path: &'a Path,
    facts: &'a dyn TsFactLookup,
    resolver: &'a dyn ImportResolution,
    workspace: &'a crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &'a dyn crate::codebase::ts_resolver::VisiblePathLookup,
    graph_files: &'a GraphFiles,
    http_route_defs: &'a [(PathBuf, String)],
    interner: &'a PathInterner,
}
