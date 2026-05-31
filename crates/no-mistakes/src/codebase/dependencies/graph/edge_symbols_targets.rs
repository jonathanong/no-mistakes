fn target_node(target: &ImportedSymbolTarget) -> (NodeId, EdgeKind) {
    match target {
        ImportedSymbolTarget::Symbol { file, symbol, kind } => (
            NodeId::Symbol {
                file: file.clone(),
                symbol: symbol.clone(),
            },
            *kind,
        ),
        ImportedSymbolTarget::Node { node, kind } => (node.clone(), *kind),
    }
}

fn namespace_target_node(target: &ImportedSymbolTarget, member: &str) -> (NodeId, EdgeKind) {
    match target {
        ImportedSymbolTarget::Symbol { file, kind, .. } => (
            NodeId::Symbol {
                file: file.clone(),
                symbol: member.to_string(),
            },
            *kind,
        ),
        ImportedSymbolTarget::Node { node, kind } => (node.clone(), *kind),
    }
}
