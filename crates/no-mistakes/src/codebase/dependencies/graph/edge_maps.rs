fn add_edge(map: &mut EdgeMap, from: NodeId, to: NodeId, kind: EdgeKind) {
    map.entry(from).or_default().push((to, kind));
}

fn normalize_nodes(nodes: &[NodeId]) -> Vec<NodeId> {
    nodes
        .iter()
        .map(|node| match node {
            NodeId::File(path) => NodeId::File(crate::codebase::ts_resolver::normalize_path(path)),
            NodeId::Symbol { file, symbol } => NodeId::Symbol {
                file: crate::codebase::ts_resolver::normalize_path(file),
                symbol: symbol.clone(),
            },
            NodeId::Module(specifier) => NodeId::Module(specifier.clone()),
            NodeId::QueueJob { queue_file, job } => NodeId::QueueJob {
                queue_file: crate::codebase::ts_resolver::normalize_path(queue_file),
                job: job.clone(),
            },
        })
        .collect()
}

/// Merge a flat list of edges into forward and reverse maps.
fn merge_edges(forward: &mut EdgeMap, reverse: &mut EdgeMap, edges: Vec<Edge>) {
    for (from, to, kind) in edges {
        forward
            .entry(from.clone())
            .or_default()
            .push((to.clone(), kind));
        reverse.entry(to).or_default().push((from, kind));
    }
}

fn remove_edges_of_kind(forward: &mut EdgeMap, reverse: &mut EdgeMap, kind: EdgeKind) {
    for edges in forward.values_mut().chain(reverse.values_mut()) {
        edges.retain(|(_, edge_kind)| *edge_kind != kind);
    }
}
