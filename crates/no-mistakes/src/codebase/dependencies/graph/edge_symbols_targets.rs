fn target_node(target: &ImportedSymbolTarget) -> (NodeId, EdgeKind) {
    match target {
        ImportedSymbolTarget::Symbol { file, symbol, kind } => (
            NodeId::symbol(file.clone(), symbol.clone()),
            *kind,
        ),
        ImportedSymbolTarget::Node { node, kind } => (node.clone(), *kind),
    }
}

fn namespace_target_node(target: &ImportedSymbolTarget, member: &str) -> (NodeId, EdgeKind) {
    match target {
        ImportedSymbolTarget::Symbol { file, kind, .. } => (
            NodeId::symbol(file.clone(), member),
            *kind,
        ),
        ImportedSymbolTarget::Node { node, kind } => (node.clone(), *kind),
    }
}

fn namespace_file_node(target: &ImportedSymbolTarget) -> (NodeId, EdgeKind) {
    match target {
        ImportedSymbolTarget::Symbol { file, kind, .. } => (NodeId::file(file.clone()), *kind),
        ImportedSymbolTarget::Node { node, kind } => (node.clone(), *kind),
    }
}
