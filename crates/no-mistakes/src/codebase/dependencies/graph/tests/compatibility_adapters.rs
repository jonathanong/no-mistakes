fn collect_route_edges(
    root: &Path,
    tsconfig: &TsConfig,
    resolver: &dyn ImportResolution,
    all_files: &[PathBuf],
    facts: Option<&dyn TsFactLookup>,
    config_options: Option<&GraphConfigOptions>,
) -> Vec<Edge> {
    let graph_files = GraphFiles::from_files(all_files.to_vec());
    super::collect_route_edges_with_graph_files(
        root, tsconfig, None, resolver, &graph_files, facts, config_options,
    )
}

fn scoped_import_map(
    imports: &[ExtractedImport],
    path: &Path,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &HashSet<PathBuf>,
) -> HashMap<String, Vec<(NodeId, EdgeKind)>> {
    let graph_files = GraphFiles::from_files(visible_files.iter().cloned().collect());
    super::scoped_import_map_with_graph_files(
        imports, path, resolver, workspace, visible_files, &graph_files,
    )
}

fn import_target(
    specifier: &str,
    kind: ImportKind,
    path: &Path,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &HashSet<PathBuf>,
) -> Option<(NodeId, EdgeKind)> {
    let graph_files = GraphFiles::from_files(visible_files.iter().cloned().collect());
    super::import_target_with_graph_files(
        specifier, kind, path, resolver, workspace, visible_files, &graph_files,
    )
}

fn resolve_imported_callee(
    callee: &str,
    imported_symbols: &HashMap<String, ImportedSymbolTarget>,
    namespace_imports: &HashMap<String, ImportedSymbolTarget>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &HashSet<PathBuf>,
) -> Option<(NodeId, EdgeKind)> {
    let graph_files = GraphFiles::from_files(visible_files.iter().cloned().collect());
    super::resolve_imported_callee_with_graph_files(
        callee,
        imported_symbols,
        namespace_imports,
        super::ReexportResolutionInputs {
            facts,
            resolver,
            workspace,
            visible_files,
            graph_files: &graph_files,
        },
    )
}
