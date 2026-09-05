fn normalize_nodes(nodes: &[NodeId]) -> Vec<NodeId> {
    nodes
        .iter()
        .map(|node| match node {
            NodeId::File(path) => {
                NodeId::file(crate::codebase::ts_resolver::normalize_path(path.as_ref()))
            }
            NodeId::Symbol { file, symbol } => NodeId::symbol(
                crate::codebase::ts_resolver::normalize_path(file),
                symbol.clone(),
            ),
            NodeId::Module(specifier) => NodeId::Module(specifier.clone()),
            NodeId::QueueJob { queue_file, job } => NodeId::queue_job(
                crate::codebase::ts_resolver::normalize_path(queue_file),
                job.clone(),
            ),
            NodeId::WorkflowJob { workflow_file, job } => NodeId::workflow_job(
                crate::codebase::ts_resolver::normalize_path(workflow_file),
                job.clone(),
            ),
            NodeId::WorkflowStep {
                workflow_file,
                job,
                step,
            } => NodeId::workflow_step(
                crate::codebase::ts_resolver::normalize_path(workflow_file),
                job.clone(),
                *step,
            ),
            NodeId::TrpcProcedure {
                router_file,
                procedure,
            } => NodeId::trpc_procedure(
                crate::codebase::ts_resolver::normalize_path(router_file),
                procedure.clone(),
            ),
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

/// Seed isolated endpoints in the forward map, then merge. Language collectors
/// historically keep unused targets as graph members even without outgoing edges.
fn merge_seeded_edges(forward: &mut EdgeMap, reverse: &mut EdgeMap, edges: Vec<Edge>) {
    for (from, to, _) in &edges {
        forward.entry(from.clone()).or_default();
        forward.entry(to.clone()).or_default();
    }
    merge_edges(forward, reverse, edges);
}

pub(crate) fn edge_index_from_maps(
    mut forward: EdgeMap,
    mut reverse: EdgeMap,
) -> EdgeIndex<NodeId, EdgeKind> {
    // Preserve the historical graph-membership boundary: only nodes present in
    // the forward map count as graph nodes. Adjacency is normalized before a
    // source-ordered flatten so canonical edge ordinals retain the exact order
    // of the former global edge comparator without a repository-wide sort.
    sort_adjacency_lists(&mut forward, &mut reverse);
    // Cache concatenated source keys so flatten sorts with slice memcmp
    // instead of rebuilding Cow parts on every compare.
    EdgeIndex::from_normalized_adjacency_maps_by_cached_source_key(
        forward,
        reverse,
        |node| (cached_node_sort_key(node), node.clone()),
    )
}

fn sort_edge_index_adjacency(index: &mut EdgeIndex<NodeId, EdgeKind>) {
    index.sort_adjacency_by_cached_key(|(node, kind)| adjacency_sort_key(node, *kind));
}
