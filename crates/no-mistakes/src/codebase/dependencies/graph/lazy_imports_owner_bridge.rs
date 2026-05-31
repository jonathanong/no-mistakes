fn should_expand_node(from: &NodeId, to: &NodeId, owner_bridge_allowed: bool) -> bool {
    !is_symbol_owner_bridge(from, to) || owner_bridge_allowed
}

fn edge_allowed(
    from: &NodeId,
    to: &NodeId,
    kind: EdgeKind,
    allowed: Option<&HashSet<EdgeKind>>,
    owner_bridge_allowed: bool,
) -> bool {
    allowed.is_none_or(|a| a.contains(&kind))
        || (is_symbol_owner_bridge(from, to) && owner_bridge_allowed)
}

fn symbol_owner_bridge_allowed(
    from: &NodeId,
    to: &NodeId,
    allowed: Option<&HashSet<EdgeKind>>,
    root_nodes: &HashSet<NodeId>,
    dynamic_import_files: &HashSet<NodeId>,
) -> bool {
    is_symbol_owner_bridge(from, to)
        && !dynamic_import_files.contains(from)
        && root_nodes.contains(from)
        && allowed.is_none_or(symbol_relationship_filter_allows_owner_bridge)
}

fn symbol_relationship_filter_allows_owner_bridge(allowed: &HashSet<EdgeKind>) -> bool {
    allowed.contains(&EdgeKind::Import)
        || allowed.contains(&EdgeKind::TypeImport)
        || allowed.contains(&EdgeKind::WorkspaceImport)
        || allowed.contains(&EdgeKind::AssetImport)
}

fn is_symbol_owner_bridge(from: &NodeId, to: &NodeId) -> bool {
    match (from, to) {
        (NodeId::File(file), NodeId::Symbol { file: symbol_file, .. })
        | (NodeId::Symbol { file: symbol_file, .. }, NodeId::File(file)) => file == symbol_file,
        _ => false,
    }
}
