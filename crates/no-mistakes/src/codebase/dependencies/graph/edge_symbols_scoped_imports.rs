fn scoped_import_map(
    imports: &[ExtractedImport],
    path: &Path,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
) -> HashMap<String, Vec<(NodeId, EdgeKind)>> {
    let mut map: HashMap<String, Vec<(NodeId, EdgeKind)>> = HashMap::new();
    for import in imports {
        let Some(scope) = &import.function_scope else {
            continue;
        };
        let Some((node, kind)) =
            import_target(&import.specifier, import.kind, path, resolver, workspace)
        else {
            continue;
        };
        map.entry(scope.clone()).or_default().push((node, kind));
    }
    for imports in map.values_mut() {
        imports.sort();
        imports.dedup();
    }
    map
}

fn import_target(
    specifier: &str,
    kind: ImportKind,
    path: &Path,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
) -> Option<(NodeId, EdgeKind)> {
    let edge_kind = match kind {
        ImportKind::Static => EdgeKind::Import,
        ImportKind::Type => EdgeKind::TypeImport,
        ImportKind::Dynamic => EdgeKind::DynamicImport,
        ImportKind::Require => EdgeKind::Require,
    };
    if let Some(target) = resolver.resolve(specifier, path) {
        let edge_kind = if is_indexable(&target) {
            edge_kind
        } else {
            EdgeKind::AssetImport
        };
        return Some((NodeId::File(target), edge_kind));
    }
    if let Some(target) = workspace.resolve_specifier_from(specifier, path) {
        return Some((NodeId::File(target), EdgeKind::WorkspaceImport));
    }
    bare_module_node(specifier).map(|node| (node, edge_kind))
}
