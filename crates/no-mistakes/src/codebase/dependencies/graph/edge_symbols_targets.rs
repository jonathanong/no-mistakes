fn target_node(target: &ImportedSymbolTarget, interner: &PathInterner) -> (NodeId, EdgeKind) {
    match target {
        ImportedSymbolTarget::Symbol { file, symbol, kind } => (
            NodeId::symbol_in(interner, file.clone(), symbol.clone()),
            *kind,
        ),
        ImportedSymbolTarget::Node { node, kind } => (node.clone(), *kind),
    }
}

fn namespace_target_node(
    target: &ImportedSymbolTarget,
    member: &str,
    interner: &PathInterner,
) -> (NodeId, EdgeKind) {
    match target {
        ImportedSymbolTarget::Symbol { file, kind, .. } => {
            (NodeId::symbol_in(interner, file.clone(), member), *kind)
        }
        ImportedSymbolTarget::Node { node, kind } => (node.clone(), *kind),
    }
}

fn namespace_file_node(
    target: &ImportedSymbolTarget,
    interner: &PathInterner,
) -> (NodeId, EdgeKind) {
    match target {
        ImportedSymbolTarget::Symbol { file, kind, .. } => {
            (NodeId::file_in(interner, file), *kind)
        }
        ImportedSymbolTarget::Node { node, kind } => (node.clone(), *kind),
    }
}
