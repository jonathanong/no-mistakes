fn roots_with_existing_queue_jobs(
    roots: &[NodeId],
    entrypoints: &[Entrypoint],
    graph: &graph::DepGraph,
) -> Vec<NodeId> {
    roots_with_existing_queue_jobs_by(roots, entrypoints, |node| graph.has_reverse_node(node))
}

fn roots_with_exported_symbol_roots(roots: &[NodeId], graph: &graph::DepGraph) -> Vec<NodeId> {
    roots_with_exported_symbol_roots_by(roots, |node| graph.dependencies_of_node(node))
}

fn roots_with_exported_symbol_roots_by<'a, F>(roots: &[NodeId], dependencies_of: F) -> Vec<NodeId>
where
    F: Fn(&NodeId) -> Option<&'a Vec<(NodeId, EdgeKind)>>,
{
    let mut seen = std::collections::HashSet::new();
    let mut expanded = Vec::new();
    for root in roots {
        push_unique_root(&mut seen, &mut expanded, root.clone());
        let NodeId::File(root_file) = root else {
            continue;
        };
        let Some(dependencies) = dependencies_of(root) else {
            continue;
        };
        for (node, _) in dependencies {
            if matches!(node, NodeId::Symbol { file, .. } if file == root_file) {
                push_unique_root(&mut seen, &mut expanded, node.clone());
            }
        }
    }
    expanded
}

fn push_unique_root(
    seen: &mut std::collections::HashSet<NodeId>,
    expanded: &mut Vec<NodeId>,
    node: NodeId,
) {
    if seen.insert(node.clone()) {
        expanded.push(node);
    }
}

fn roots_with_existing_queue_jobs_by<F>(
    roots: &[NodeId],
    entrypoints: &[Entrypoint],
    has_reverse_node: F,
) -> Vec<NodeId>
where
    F: Fn(&NodeId) -> bool,
{
    let mut roots = roots.to_vec();
    for entrypoint in entrypoints {
        let Some(symbol) = &entrypoint.symbol else {
            continue;
        };
        if matches!(entrypoint.node, NodeId::Module(_)) {
            continue;
        }
        let queue_job = NodeId::QueueJob {
            queue_file: entrypoint.file.clone(),
            job: symbol.clone(),
        };
        if has_reverse_node(&queue_job) {
            roots.push(queue_job);
        }
    }
    roots
}
