fn bfs<'a, A>(
    starts: &'a [NodeId],
    edges: &'a FxHashMap<NodeId, A>,
    max_depth: Option<usize>,
    allowed: Option<&HashSet<EdgeKind>>,
) -> Vec<NodeEntry>
where
    A: AsRef<[(NodeId, EdgeKind)]>,
{
    bfs_with_file_universe(starts, edges, max_depth, allowed, None)
}

fn bfs_in_file_universe<'a, A>(
    starts: &'a [NodeId],
    edges: &'a FxHashMap<NodeId, A>,
    max_depth: Option<usize>,
    allowed: Option<&HashSet<EdgeKind>>,
    file_universe: &crate::fx::PathSet,
) -> Vec<NodeEntry>
where
    A: AsRef<[(NodeId, EdgeKind)]>,
{
    bfs_with_file_universe(starts, edges, max_depth, allowed, Some(file_universe))
}

fn bfs_with_file_universe<'a, A>(
    starts: &'a [NodeId],
    edges: &'a FxHashMap<NodeId, A>,
    max_depth: Option<usize>,
    allowed: Option<&HashSet<EdgeKind>>,
    file_universe: Option<&crate::fx::PathSet>,
) -> Vec<NodeEntry>
where
    A: AsRef<[(NodeId, EdgeKind)]>,
{
    // Working sets borrow NodeIds from `starts` / `edges`. Clone only when
    // emitting an owned NodeEntry — each extra HashSet/Vec insert would
    // otherwise bump every interned Arc.
    let mut visited: FxHashSet<&NodeId> = fx_set();
    let mut queue: VecDeque<(&NodeId, usize)> = VecDeque::new();
    let mut result: Vec<NodeEntry> = Vec::new();
    let mut result_idx: FxHashMap<&NodeId, usize> = fx_map();
    let mut dynamic_import_files: FxHashSet<&NodeId> = fx_set();

    for start in starts {
        if file_universe.is_some_and(|universe| !start.is_in_file_universe(universe)) {
            continue;
        }
        if visited.insert(start) {
            queue.push_back((start, 0));
        }
    }
    let root_nodes: FxHashSet<NodeId> = starts.iter().cloned().collect();

    let mut check_counter = 0u32;
    while let Some((node, depth)) = queue.pop_front().filter(|_| {
        check_counter += 1;
        !check_counter.is_multiple_of(256) || crate::invocation::check_timeout().is_ok()
    }) {
        if max_depth.is_some_and(|max| depth >= max) {
            continue;
        }

        if let Some(neighbors) = edges.get(node) {
            let neighbors: &'a [(NodeId, EdgeKind)] = neighbors.as_ref();
            for (neighbor, kind) in neighbors {
                if file_universe.is_some_and(|universe| !neighbor.is_in_file_universe(universe)) {
                    continue;
                }
                let from_is_dynamic_import_file = dynamic_import_files.contains(node);
                if from_is_dynamic_import_file && matches!(neighbor, NodeId::Symbol { .. }) {
                    continue;
                }
                let owner_bridge_allowed = symbol_owner_bridge_allowed(
                    node,
                    neighbor,
                    &root_nodes,
                    from_is_dynamic_import_file,
                );
                if is_symbol_owner_bridge(node, neighbor) && !owner_bridge_allowed {
                    continue;
                }
                if !edge_allowed(node, neighbor, *kind, allowed, owner_bridge_allowed) {
                    continue;
                }

                if visited.insert(neighbor) {
                    let next_depth = depth + 1;
                    if should_emit_node(node, neighbor, *kind, allowed, owner_bridge_allowed) {
                        let index = result.len();
                        result.push(NodeEntry {
                            node: neighbor.clone(),
                            depth: next_depth,
                            via: vec![*kind],
                        });
                        result_idx.insert(neighbor, index);
                    }
                    if *kind == EdgeKind::DynamicImport && matches!(neighbor, NodeId::File(_)) {
                        dynamic_import_files.insert(neighbor);
                    }
                    if should_expand_node(node, neighbor, owner_bridge_allowed) {
                        queue.push_back((neighbor, next_depth));
                    }
                } else if let Some(&index) = result_idx.get(neighbor) {
                    add_via_kind(&mut result[index], *kind);
                }
            }
        }
    }

    result
}
