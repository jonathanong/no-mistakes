fn bfs(
    starts: &[NodeId],
    edges: &EdgeMap,
    max_depth: Option<usize>,
    allowed: Option<&HashSet<EdgeKind>>,
) -> Vec<NodeEntry> {
    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut queue: VecDeque<(NodeId, usize)> = VecDeque::new();
    let mut result: Vec<NodeEntry> = Vec::new();
    let mut result_idx: HashMap<NodeId, usize> = HashMap::new();
    let mut dynamic_import_files: HashSet<NodeId> = HashSet::new();

    for start in starts {
        if !visited.contains(start) {
            visited.insert(start.clone());
            queue.push_back((start.clone(), 0));
        }
    }
    let root_nodes: HashSet<NodeId> = starts.iter().cloned().collect();

    while let Some((node, depth)) = queue.pop_front() {
        if max_depth.is_some_and(|max| depth >= max) {
            continue;
        }

        if let Some(neighbors) = edges.get(&node) {
            for (neighbor, kind) in neighbors {
                if dynamic_import_files.contains(&node)
                    && matches!(neighbor, NodeId::Symbol { .. })
                {
                    continue;
                }
                let owner_bridge_allowed =
                    symbol_owner_bridge_allowed(&node, neighbor, &root_nodes, &dynamic_import_files);
                if is_symbol_owner_bridge(&node, neighbor) && !owner_bridge_allowed {
                    continue;
                }
                if !edge_allowed(&node, neighbor, *kind, allowed, owner_bridge_allowed) {
                    continue;
                }

                if !visited.contains(neighbor) {
                    visited.insert(neighbor.clone());
                    let next_depth = depth + 1;
                    if should_emit_node(&node, neighbor, *kind, allowed, owner_bridge_allowed) {
                        let index = result.len();
                        result.push(NodeEntry {
                            node: neighbor.clone(),
                            depth: next_depth,
                            via: vec![*kind],
                        });
                        result_idx.insert(neighbor.clone(), index);
                    }
                    if *kind == EdgeKind::DynamicImport && matches!(neighbor, NodeId::File(_)) {
                        dynamic_import_files.insert(neighbor.clone());
                    }
                    if should_expand_node(&node, neighbor, owner_bridge_allowed) {
                        queue.push_back((neighbor.clone(), next_depth));
                    }
                } else if let Some(&index) = result_idx.get(neighbor) {
                    add_via_kind(&mut result[index], *kind);
                }
            }
        }
    }

    result
}
