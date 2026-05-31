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
    root: &Path,
    tsconfig: &TsConfig,
    max_depth: Option<usize>,
    graph_files: &GraphFiles,
    allowed: Option<&HashSet<EdgeKind>>,
) -> Vec<NodeEntry> {
    let resolver = ImportResolver::new(tsconfig).with_visible(&graph_files.visible);
    let workspace =
        crate::codebase::workspaces::load_from_files(root, graph_files.all()).unwrap_or_default();

    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut frontier: Vec<NodeId> = Vec::new();
    let mut result: Vec<NodeEntry> = Vec::new();
    let mut result_idx: HashMap<NodeId, usize> = HashMap::new();

    for root in roots {
        if !visited.contains(root) {
            visited.insert(root.clone());
            frontier.push(root.clone());
        }
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
                    import_neighbors(path, &resolver, &workspace, graph_files, allowed),
                )
            })
            .collect();
        // ⚡ Bolt: Use `sort_by_cached_key` instead of `sort_by_key` to avoid repeatedly calling
        // `node_sort_key` (which involves allocation and formatting) during the sort operations.
        expanded.sort_by_cached_key(|(node, _)| node_sort_key(node));

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
        if !visited.contains(s) {
            visited.insert(s.clone());
            queue.push_back((s.clone(), 0));
        }
    }

    while let Some((node, depth)) = queue.pop_front() {
        if let Some(max) = max_depth {
            if depth >= max {
                continue;
            }
        }

        if let Some(neighbors) = edges.get(&node) {
            for (neighbor, kind) in neighbors {
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
