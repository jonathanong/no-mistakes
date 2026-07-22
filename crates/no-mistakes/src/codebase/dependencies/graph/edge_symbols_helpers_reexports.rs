#[derive(Clone, Copy)]
struct ReexportResolutionInputs<'a> {
    facts: &'a dyn TsFactLookup,
    resolver: &'a dyn ImportResolution,
    workspace: &'a crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &'a HashSet<PathBuf>,
    graph_files: &'a GraphFiles,
}

fn resolve_imported_callee_with_graph_files(
    callee: &str,
    imported_symbols: &HashMap<String, ImportedSymbolTarget>,
    namespace_imports: &HashMap<String, ImportedSymbolTarget>,
    inputs: ReexportResolutionInputs<'_>,
) -> Option<(NodeId, EdgeKind)> {
    if let Some(target) = imported_symbols.get(callee) {
        return Some(target_node(target));
    }
    if let Some(target) = namespace_imports.get(callee) {
        return Some(namespace_file_node(target));
    }
    let (namespace, member) = callee.split_once('.')?;
    if let Some(target) = namespace_imports.get(namespace) {
        return Some(namespace_target_node(target, member));
    }
    let ImportedSymbolTarget::Symbol {
        file: barrel,
        symbol: imported,
        kind,
    } = imported_symbols.get(namespace)?
    else {
        return None;
    };
    resolve_reexported_namespace_member(
        barrel,
        imported,
        member,
        *kind,
        ReexportNamespaceInputs {
            facts: inputs.facts,
            resolver: inputs.resolver,
            workspace: inputs.workspace,
            visible_files: inputs.visible_files,
            graph_files: inputs.graph_files,
        },
    )
}
