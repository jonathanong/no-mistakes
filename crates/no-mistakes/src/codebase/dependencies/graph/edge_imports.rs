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
    interner: &PathInterner,
) -> Vec<Edge> {
    parsed_imports
        .par_iter()
        .flat_map_iter(|(path, facts, reachable)| {
            facts
                .imports
                .iter()
                .filter_map(|imp| {
                    let kind = graph_edge_kind_for_extracted_import(imp, facts, reachable)?;
                    let classification =
                        resolver.classify_import(&imp.specifier, path, workspace, graph_files);
                    let target = if kind == EdgeKind::RequireResolve {
                        classification
                            .preferred_path()
                            .and_then(|target| graph_files.visible_path(target))
                    } else {
                        classification
                            .resolver_path()
                            .and_then(|target| graph_files.visible_path(target))
                            // Workspace exports may resolve through an invisible node_modules symlink.
                            // Keep the literal dynamic edge kind while using the prepared visible target.
                            .or_else(|| {
                                matches!(imp.kind, ImportKind::Dynamic)
                                    .then(|| {
                                        classification
                                            .workspace_path()
                                            .and_then(|target| graph_files.visible_path(target))
                                    })
                                    .flatten()
                            })
                    };
                    if let Some(target) = target {
                        return (is_indexable(target) || kind == EdgeKind::RequireResolve).then(
                            || {
                                (
                                    NodeId::file_in(interner, *path),
                                    NodeId::file_in(interner, target),
                                    kind,
                                )
                            },
                        );
                    }
                    if classification.is_unresolved_external() {
                        return bare_module_node_in(interner, &imp.specifier)
                            .map(|module| (NodeId::file_in(interner, *path), module, kind));
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
    interner: &PathInterner,
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
                            NodeId::file_in(interner, *path),
                            NodeId::file_in(interner, target),
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
    interner: &PathInterner,
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
                .filter(|imp| graph_edge_kind_for_extracted_import(imp, facts, reachable).is_some())
                .filter(|imp| !matches!(imp.kind, ImportKind::RequireResolve))
                .filter_map(|imp| {
                    let spec = &imp.specifier;
                    if spec.starts_with('.') {
                        return None;
                    }
                    resolver
                        .classify_import(spec, path, workspace, graph_files)
                        .workspace_path()
                        .and_then(|entry| graph_files.visible_path(entry))
                        .map(|entry| {
                            let kind = match imp.kind {
                                ImportKind::Type => EdgeKind::WorkspaceTypeImport,
                                ImportKind::RequireResolve => EdgeKind::RequireResolve,
                                _ => EdgeKind::WorkspaceImport,
                            };
                            (
                                NodeId::file_in(interner, *path),
                                NodeId::file_in(interner, entry),
                                kind,
                            )
                        })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
