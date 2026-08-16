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
    resolver: &dyn ImportResolution,
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
                        let target = graph_files.visible_path(target)?;
                        return (is_indexable(target) || kind == EdgeKind::RequireResolve).then(|| {
                            (
                                NodeId::file((*path).clone()),
                                NodeId::file(target),
                                kind,
                            )
                        });
                    }
                    if classification.is_unresolved_external() {
                        return bare_module_node(&imp.specifier)
                            .map(|module| (NodeId::file((*path).clone()), module, kind));
                    }
                    None
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn collect_asset_edges(
    parsed_imports: &ParsedImports<'_>,
    resolver: &dyn ImportResolution,
    graph_files: &GraphFiles,
) -> Vec<Edge> {
    parsed_imports
        .par_iter()
        .flat_map_iter(|(path, facts, reachable)| {
            facts
                .imports
                .iter()
                .filter(|imp| import_is_reachable(imp, facts, reachable))
                .filter(|imp| !matches!(imp.kind, ImportKind::Type | ImportKind::RequireResolve))
                .filter(|imp| imp.specifier.starts_with('.') || imp.specifier.starts_with('/'))
                .filter_map(|imp| {
                    resolver.resolve(&imp.specifier, path).and_then(|target| {
                        let target = graph_files.visible_path(&target)?;
                        if is_indexable(target) {
                            return None;
                        }
                        Some((
                            NodeId::file((*path).clone()),
                            NodeId::file(target),
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
    resolver: &dyn ImportResolution,
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
                        .and_then(|entry| graph_files.visible_path(entry))
                        .map(|entry| {
                            let kind = match imp.kind {
                                ImportKind::Type => EdgeKind::WorkspaceTypeImport,
                                ImportKind::RequireResolve => EdgeKind::RequireResolve,
                                _ => EdgeKind::WorkspaceImport,
                            };
                            (
                                NodeId::file((*path).clone()),
                                NodeId::file(entry),
                                kind,
                            )
                        })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
