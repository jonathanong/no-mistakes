fn collect_parsed_imports_from_facts<'a>(
    files: &'a [PathBuf],
    facts: &'a dyn TsFactLookup,
) -> ParsedImports<'a> {
    files
        .par_iter()
        .filter_map(|path| {
            facts.get_ts_facts(path).map(|file_facts| {
                let reachable = reachable_function_scopes(file_facts);
                (path, file_facts, reachable)
            })
        })
        .collect()
}

fn collect_import_edges(
    parsed_imports: &ParsedImports<'_>,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    graph_files: &GraphFiles,
) -> Vec<Edge> {
    parsed_imports
        .par_iter()
        .flat_map_iter(|(path, facts, reachable)| {
            facts
                .imports
                .iter()
                .filter(|imp| import_is_reachable(imp, facts, reachable))
                .filter_map(|imp| {
                    let kind = edge_kind_for_import(imp);
                    let classification = resolver.classify_import(
                        &imp.specifier,
                        path,
                        workspace,
                        graph_files.visible(),
                    );
                    if let Some(target) = classification.resolver_path() {
                        return (graph_files.is_visible(target) && is_indexable(target)).then(|| {
                            (
                                NodeId::File((*path).clone()),
                                NodeId::File(target.to_path_buf()),
                                kind,
                            )
                        });
                    }
                    if classification.is_unresolved_external() {
                        return bare_module_node(&imp.specifier)
                            .map(|module| (NodeId::File((*path).clone()), module, kind));
                    }
                    None
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
        .flat_map_iter(|(path, facts, reachable)| {
            facts
                .imports
                .iter()
                .filter(|imp| import_is_reachable(imp, facts, reachable))
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
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    graph_files: &GraphFiles,
) -> Vec<Edge> {
    if workspace.packages.is_empty() {
        return vec![];
    }

    parsed_imports
        .par_iter()
        .flat_map_iter(|(path, facts, reachable)| {
            facts
                .imports
                .iter()
                .filter(|imp| import_is_reachable(imp, facts, reachable))
                .filter_map(|imp| {
                    let spec = &imp.specifier;
                    if spec.starts_with('.') {
                        return None;
                    }
                    resolver
                        .classify_import(spec, path, workspace, graph_files.visible())
                        .workspace_path()
                        .filter(|entry| graph_files.is_visible(entry))
                        .map(|entry| {
                            (
                                NodeId::File((*path).clone()),
                                NodeId::File(entry.to_path_buf()),
                                EdgeKind::WorkspaceImport,
                            )
                        })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
