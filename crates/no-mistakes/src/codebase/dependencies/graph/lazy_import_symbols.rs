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

fn bfs_skipping_symbol_owner_files<A>(
    starts: &[NodeId],
    edges: &FxHashMap<NodeId, A>,
    max_depth: Option<usize>,
    allowed: Option<&HashSet<EdgeKind>>,
) -> Vec<NodeEntry>
where
    A: AsRef<[(NodeId, EdgeKind)]>,
{
    let mut visited: HashSet<(NodeId, Option<PathBuf>)> = HashSet::new();
    let mut queue: VecDeque<(NodeId, usize, Option<PathBuf>)> = VecDeque::new();
    let mut result: Vec<NodeEntry> = Vec::new();
    let mut result_idx: FxHashMap<NodeId, usize> = FxHashMap::default();
    let symbol_importer_files_by_owner = symbol_importer_files_by_owner(edges);
    let root_symbols: HashSet<(PathBuf, std::sync::Arc<str>)> = starts
        .iter()
        .filter_map(|node| {
            if let NodeId::Symbol { file, symbol } = node {
                Some((file.to_path_buf(), symbol.clone()))
            } else {
                None
            }
        })
        .collect();

    for s in starts {
        let state = (s.clone(), None);
        if !visited.contains(&state) {
            visited.insert(state);
            queue.push_back((s.clone(), 0, None));
        }
    }

    let mut check_counter = 0u32;
    while let Some((node, depth, owner_context)) = queue.pop_front().filter(|_| {
        check_counter += 1;
        !check_counter.is_multiple_of(256) || crate::invocation::check_timeout().is_ok()
    }) {
        if max_depth.is_some_and(|max| depth >= max) {
            continue;
        }

        if let Some(neighbors) = edges.get(&node) {
            for (neighbor, kind) in neighbors.as_ref() {
                if let (
                    NodeId::Symbol {
                        file: owner,
                        symbol,
                    },
                    NodeId::File(neighbor_file),
                ) = (&node, neighbor)
                {
                    if neighbor_file == owner
                        && root_symbols.contains(&(owner.to_path_buf(), symbol.clone()))
                    {
                        continue;
                    }
                }
                if let (Some(owner), NodeId::File(importer)) = (&owner_context, neighbor) {
                    if symbol_importer_files_by_owner
                        .get(owner.as_path())
                        .is_some_and(|files| files.contains(importer.as_ref()))
                    {
                        continue;
                    }
                }
                if !allowed.is_none_or(|a| a.contains(kind)) {
                    continue;
                }

                let next_depth = depth + 1;
                let next_owner_context = match (&node, neighbor) {
                    (NodeId::Symbol { file: owner, .. }, NodeId::File(neighbor_file))
                        if neighbor_file == owner =>
                    {
                        Some(owner.to_path_buf())
                    }
                    _ => None,
                };
                if visited.insert((neighbor.clone(), next_owner_context.clone())) {
                    if let Some(&idx) = result_idx.get(neighbor) {
                        add_via_kind(&mut result[idx], *kind);
                    } else {
                        let idx = result.len();
                        result.push(NodeEntry {
                            node: neighbor.clone(),
                            depth: next_depth,
                            via: vec![*kind],
                        });
                        result_idx.insert(neighbor.clone(), idx);
                    }
                    queue.push_back((neighbor.clone(), next_depth, next_owner_context));
                } else if let Some(&idx) = result_idx.get(neighbor) {
                    add_via_kind(&mut result[idx], *kind);
                }
            }
        }
    }

    result
}

fn symbol_importer_files_by_owner<A>(
    edges: &FxHashMap<NodeId, A>,
) -> HashMap<PathBuf, HashSet<PathBuf>>
where
    A: AsRef<[(NodeId, EdgeKind)]>,
{
    let mut map: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
    for (target, importers) in edges {
        let NodeId::Symbol { file: owner, .. } = target else {
            continue;
        };
        let files = map.entry(owner.to_path_buf()).or_default();
        for (importer, _) in importers.as_ref() {
            match importer {
                NodeId::File(file) | NodeId::Symbol { file, .. } => {
                    files.insert(file.to_path_buf());
                }
                _ => {}
            }
        }
    }
    map
}
