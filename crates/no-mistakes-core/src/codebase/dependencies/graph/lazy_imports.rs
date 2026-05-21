
fn push_unvisited_symbol_pair(
    visited_pairs: &mut HashSet<(PathBuf, String)>,
    queue: &mut VecDeque<(PathBuf, String)>,
    pair: (PathBuf, String),
) {
    if visited_pairs.insert(pair.clone()) {
        queue.push_back(pair);
    }
}

/// Demand-driven import traversal used by `dependencies --relationship import`.
/// It parses only roots and files reached through static import edges.
pub fn lazy_import_deps_of(
    roots: &[NodeId],
    root: &Path,
    tsconfig: &TsConfig,
    max_depth: Option<usize>,
) -> Result<Vec<NodeEntry>> {
    let graph_files = GraphFiles::discover(root);
    Ok(lazy_import_deps_of_with_files(
        roots,
        root,
        tsconfig,
        max_depth,
        &graph_files,
        None,
    ))
}

pub(crate) fn lazy_import_deps_of_with_files(
    roots: &[NodeId],
    _root: &Path,
    tsconfig: &TsConfig,
    max_depth: Option<usize>,
    graph_files: &GraphFiles,
    allowed: Option<&HashSet<EdgeKind>>,
) -> Vec<NodeEntry> {
    let ts_ex = ImportExtractor::for_typescript().expect("typescript import extractor builds");
    let tsx_ex = ImportExtractor::for_tsx().expect("tsx import extractor builds");
    let resolver = ImportResolver::new(tsconfig).with_visible(&graph_files.visible);

    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut frontier: Vec<NodeId> = Vec::new();
    let mut result: Vec<NodeEntry> = Vec::new();
    let mut result_idx: HashMap<NodeId, usize> = HashMap::new();

    for root in roots {
        visited.insert(root.clone());
        frontier.push(root.clone());
    }

    let mut depth = 0;
    while !frontier.is_empty() {
        if let Some(max) = max_depth {
            if depth >= max {
                break;
            }
        }

        let mut expanded: Vec<(NodeId, Vec<(NodeId, EdgeKind)>)> = frontier
            .par_iter()
            .map(|node| {
                let Some(path) = node.as_file() else {
                    return (node.clone(), Vec::new());
                };
                if !graph_files.is_visible(path) || !is_indexable(path) {
                    return (node.clone(), Vec::new());
                }
                (
                    node.clone(),
                    import_neighbors(path, &resolver, &ts_ex, &tsx_ex, graph_files, allowed),
                )
            })
            .collect();
        expanded.sort_by_key(|(node, _)| node_sort_key(node));

        let next_depth = depth + 1;
        let mut next_frontier = Vec::new();
        for (_node, neighbors) in expanded {
            for (neighbor, kind) in neighbors {
                if visited.insert(neighbor.clone()) {
                    let idx = result.len();
                    result.push(NodeEntry {
                        node: neighbor.clone(),
                        depth: next_depth,
                        via: vec![kind],
                    });
                    result_idx.insert(neighbor.clone(), idx);
                    next_frontier.push(neighbor);
                } else {
                    if let Some(&idx) = result_idx.get(&neighbor) {
                        add_via_kind(&mut result[idx], kind);
                    }
                }
            }
        }
        frontier = next_frontier;
        depth = next_depth;
    }

    result
}

fn import_neighbors(
    path: &Path,
    resolver: &ImportResolver<'_>,
    ts_ex: &ImportExtractor,
    tsx_ex: &ImportExtractor,
    graph_files: &GraphFiles,
    allowed: Option<&HashSet<EdgeKind>>,
) -> Vec<(NodeId, EdgeKind)> {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(_) => return Vec::new(),
    };
    let extractor = if is_tsx_file(path) { tsx_ex } else { ts_ex };
    let mut neighbors: Vec<(NodeId, EdgeKind)> = extractor
        .extract(&source)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|imp| {
            resolver
                .resolve(&imp.specifier, path)
                .filter(|target| graph_files.is_visible(target))
                .map(|target| (NodeId::File(target), edge_kind_for_import(&imp)))
        })
        .filter(|(_, kind)| {
            if let Some(allowed) = allowed {
                allowed.contains(kind)
            } else {
                true
            }
        })
        .collect();
    neighbors.sort_by_key(|(node, kind)| (node_sort_key(node), *kind as u8));
    neighbors
}

fn push_route_ref_edge(edges: &mut Vec<Edge>, source: &Path, target: &Path) {
    edges.push((
        NodeId::File(source.to_path_buf()),
        NodeId::File(target.to_path_buf()),
        EdgeKind::RouteRef,
    ));
}

fn add_distinct_worker_file_edges(
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
    worker_file: &PathBuf,
    processor_file: &PathBuf,
    queue_job: &NodeId,
) {
    if worker_file != processor_file {
        add_edge(
            forward,
            queue_job.clone(),
            NodeId::File(worker_file.clone()),
            EdgeKind::QueueWorker,
        );
        add_edge(
            reverse,
            NodeId::File(worker_file.clone()),
            queue_job.clone(),
            EdgeKind::QueueWorker,
        );
    }
}

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

    for s in starts {
        visited.insert(s.clone());
        queue.push_back((s.clone(), 0));
    }

    while let Some((node, depth)) = queue.pop_front() {
        if let Some(max) = max_depth {
            if depth >= max {
                continue;
            }
        }

        if let Some(neighbors) = edges.get(&node) {
            for (neighbor, kind) in neighbors {
                if let Some(allowed) = allowed {
                    if !allowed.contains(kind) {
                        continue;
                    }
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


