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
            NodeId::WorkflowJob { workflow_file, job } => NodeId::WorkflowJob {
                workflow_file: crate::codebase::ts_resolver::normalize_path(workflow_file),
                job: job.clone(),
            },
            NodeId::WorkflowStep {
                workflow_file,
                job,
                step,
            } => NodeId::WorkflowStep {
                workflow_file: crate::codebase::ts_resolver::normalize_path(workflow_file),
                job: job.clone(),
                step: *step,
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
            edge_kind_rank(left.kind),
        )
            .cmp(&(
                node_sort_key(&right.from),
                &right.from,
                node_sort_key(&right.to),
                &right.to,
                edge_kind_rank(right.kind),
            ))
    })
}

fn sort_edge_index_adjacency(index: &mut EdgeIndex<NodeId, EdgeKind>) {
    index.sort_adjacency_by(|(left_node, left_kind), (right_node, right_kind)| {
        (node_sort_key(left_node), left_node, edge_kind_rank(*left_kind)).cmp(&(
            node_sort_key(right_node),
            right_node,
            edge_kind_rank(*right_kind),
        ))
    });
}
