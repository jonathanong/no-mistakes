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

fn edge_index_from_maps(mut forward: EdgeMap, mut reverse: EdgeMap) -> EdgeIndex<NodeId, EdgeKind> {
    // Preserve the historical graph-membership boundary: only nodes present in
    // the forward map count as graph nodes.
    sort_adjacency_lists(&mut forward, &mut reverse);
    EdgeIndex::from_adjacency_maps_by(forward, reverse, |left, right| {
        (
            node_sort_key(&left.from),
            &left.from,
            node_sort_key(&left.to),
            &left.to,
            left.kind as u8,
        )
            .cmp(&(
                node_sort_key(&right.from),
                &right.from,
                node_sort_key(&right.to),
                &right.to,
                right.kind as u8,
            ))
    })
}

fn sort_edge_index_adjacency(index: &mut EdgeIndex<NodeId, EdgeKind>) {
    index.sort_adjacency_by(|(left_node, left_kind), (right_node, right_kind)| {
        (node_sort_key(left_node), left_node, *left_kind as u8).cmp(&(
            node_sort_key(right_node),
            right_node,
            *right_kind as u8,
        ))
    });
}
