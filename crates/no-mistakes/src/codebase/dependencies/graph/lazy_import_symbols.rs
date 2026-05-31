fn push_unvisited_symbol_pair(
    visited_pairs: &mut HashSet<(PathBuf, String)>,
    queue: &mut VecDeque<(PathBuf, String)>,
    pair: (PathBuf, String),
) {
    if !visited_pairs.contains(&pair) {
        visited_pairs.insert(pair.clone());
        queue.push_back(pair);
    }
}

fn bfs_skipping_initial_symbol_owner_files(
    starts: &[NodeId],
    edges: &EdgeMap,
    max_depth: Option<usize>,
    allowed: Option<&HashSet<EdgeKind>>,
) -> Vec<NodeEntry> {
    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut queue: VecDeque<(NodeId, usize)> = VecDeque::new();
    let mut result: Vec<NodeEntry> = Vec::new();
    let mut result_idx: HashMap<NodeId, usize> = HashMap::new();

    for s in starts {
        if !visited.contains(s) {
            visited.insert(s.clone());
            queue.push_back((s.clone(), 0));
        }
    }

    while let Some((node, depth)) = queue.pop_front() {
        if max_depth.is_some_and(|max| depth >= max) {
            continue;
        }

        if let Some(neighbors) = edges.get(&node) {
            for (neighbor, kind) in neighbors {
                if depth == 0 {
                    if let (
                        NodeId::Symbol { file: owner, .. },
                        NodeId::File(neighbor_file),
                    ) = (&node, neighbor)
                    {
                        if neighbor_file == owner {
                            continue;
                        }
                    }
                }
                if !allowed.is_none_or(|a| a.contains(kind)) {
                    continue;
                }

                if visited.insert(neighbor.clone()) {
                    let next_depth = depth + 1;
                    let idx = result.len();
                    result.push(NodeEntry {
                        node: neighbor.clone(),
                        depth: next_depth,
                        via: vec![*kind],
                    });
                    result_idx.insert(neighbor.clone(), idx);
                    queue.push_back((neighbor.clone(), next_depth));
                } else if let Some(&idx) = result_idx.get(neighbor) {
                    add_via_kind(&mut result[idx], *kind);
                }
            }
        }
    }

    result
}
