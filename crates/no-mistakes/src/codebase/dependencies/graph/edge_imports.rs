fn collect_parsed_imports_from_facts<'a>(
    files: &'a [PathBuf],
    facts: &'a TsFactMap,
) -> ParsedImports<'a> {
    files
        .par_iter()
        .filter_map(|path| {
            facts.get(path).map(|file_facts| (path, file_facts))
        })
        .collect()
}

fn collect_import_edges(
    parsed_imports: &ParsedImports<'_>,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
    graph_files: &GraphFiles,
) -> Vec<Edge> {
    parsed_imports
        .par_iter()
        .flat_map_iter(|(path, facts)| {
            let reachable = reachable_function_scopes(facts);
            facts
                .imports
                .iter()
                .filter(|imp| import_is_reachable(imp, facts, &reachable))
                .filter_map(|imp| {
                    let kind = edge_kind_for_import(imp);
                    if let Some(target) = resolver.resolve(&imp.specifier, path) {
                        if !graph_files.is_visible(&target) || !is_indexable(&target) {
                            return None;
                        }
                        return Some((NodeId::File((*path).clone()), NodeId::File(target), kind));
                    }
                    if workspace.resolve_specifier_from(&imp.specifier, path).is_some() {
                        return None;
                    }
                    bare_module_node(&imp.specifier)
                        .map(|module| (NodeId::File((*path).clone()), module, kind))
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn collect_asset_edges(
    parsed_imports: &ParsedImports<'_>,
    resolver: &ImportResolver<'_>,
    graph_files: &GraphFiles,
) -> Vec<Edge> {
    parsed_imports
        .par_iter()
        .flat_map_iter(|(path, facts)| {
            let reachable = reachable_function_scopes(facts);
            facts
                .imports
                .iter()
                .filter(|imp| import_is_reachable(imp, facts, &reachable))
                .filter(|imp| imp.specifier.starts_with('.') || imp.specifier.starts_with('/'))
                .filter_map(|imp| {
                    resolver.resolve(&imp.specifier, path).and_then(|target| {
                        if !graph_files.is_visible(&target) || is_indexable(&target) {
                            return None;
                        }
                        Some((
                            NodeId::File((*path).clone()),
                            NodeId::File(target),
                            EdgeKind::AssetImport,
                        ))
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn collect_workspace_edges(
    parsed_imports: &ParsedImports<'_>,
    _resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
    graph_files: &GraphFiles,
) -> Vec<Edge> {
    if workspace.packages.is_empty() {
        return vec![];
    }

    parsed_imports
        .par_iter()
        .flat_map_iter(|(path, facts)| {
            let reachable = reachable_function_scopes(facts);
            facts
                .imports
                .iter()
                .filter(|imp| import_is_reachable(imp, facts, &reachable))
                .filter_map(|imp| {
                    let spec = &imp.specifier;
                    if spec.starts_with('.') {
                        return None;
                    }
                    workspace.resolve_specifier_from(spec, path).and_then(|entry| {
                        if !graph_files.is_visible(&entry) {
                            return None;
                        }
                        Some((
                            NodeId::File((*path).clone()),
                            NodeId::File(entry),
                            EdgeKind::WorkspaceImport,
                        ))
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
