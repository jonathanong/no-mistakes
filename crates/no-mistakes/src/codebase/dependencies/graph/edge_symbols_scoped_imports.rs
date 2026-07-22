fn scoped_import_map_with_graph_files(
    imports: &[ExtractedImport],
    path: &Path,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &HashSet<PathBuf>,
    graph_files: &GraphFiles,
) -> HashMap<String, Vec<(NodeId, EdgeKind)>> {
    const TOP_LEVEL_SIDE_EFFECT_SCOPE: &str = "";
    let mut map: HashMap<String, Vec<(NodeId, EdgeKind)>> = HashMap::new();
    for import in imports {
        let scope = if let Some(scope) = &import.function_scope {
            scope.as_str()
        } else if import.side_effect_only {
            TOP_LEVEL_SIDE_EFFECT_SCOPE
        } else {
            continue;
        };
        let Some((node, kind)) = import_target_with_graph_files(
            &import.specifier,
            import.kind,
            path,
            resolver,
            workspace,
            visible_files,
            graph_files,
        ) else {
            continue;
        };
        map.entry(scope.to_string()).or_default().push((node, kind));
    }
    for imports in map.values_mut() {
        imports.sort();
        imports.dedup();
    }
    map
}

fn import_target_with_graph_files(
    specifier: &str,
    kind: ImportKind,
    path: &Path,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &HashSet<PathBuf>,
    graph_files: &GraphFiles,
) -> Option<(NodeId, EdgeKind)> {
    let edge_kind = match kind {
        ImportKind::Static => EdgeKind::Import,
        ImportKind::Type => EdgeKind::TypeImport,
        ImportKind::Dynamic => EdgeKind::DynamicImport,
        ImportKind::Require | ImportKind::RequireResolve => EdgeKind::Require,
    };
    if let Some(target) = resolver.resolve(specifier, path) {
        let target = graph_files.visible_path(&target)?;
        let edge_kind = if is_indexable(target) {
            edge_kind
        } else {
            EdgeKind::AssetImport
        };
        return Some((NodeId::File(target.to_path_buf()), edge_kind));
    }
    if let Some(target) =
        workspace.resolve_specifier_from_file_visible(specifier, path, visible_files)
    {
        let target = graph_files.visible_path(&target)?;
        return Some((NodeId::File(target.to_path_buf()), EdgeKind::WorkspaceImport));
    }
    if workspace.recognizes_specifier_from(specifier, path) {
        return None;
    }
    bare_module_node(specifier).map(|node| (node, edge_kind))
}
