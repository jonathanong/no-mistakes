pub(crate) fn lazy_import_deps_of_with_files_facts_workspace_and_resolution_cache(
    input: LazyImportBuild<'_>,
) -> (Vec<NodeEntry>, Vec<(PathBuf, TsFileFacts)>) {
    let LazyImportBuild {
        roots,
        tsconfig,
        max_depth,
        graph_files,
        allowed,
        facts,
        workspace,
        import_resolution_cache,
    } = input;
    let resolver = ImportResolver::new(tsconfig).with_visible(&graph_files.visible);
    let resolver = match import_resolution_cache {
        Some(cache) => resolver.with_shared_cache(cache),
        None => resolver,
    };
    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut frontier: Vec<NodeId> = Vec::new();
    let mut result: Vec<NodeEntry> = Vec::new();
    let mut result_idx: HashMap<NodeId, usize> = HashMap::new();
    let mut collected_facts = Vec::new();

    for root in roots {
        if !visited.contains(root) {
            visited.insert(root.clone());
            frontier.push(root.clone());
        }
    }
    let root_nodes: HashSet<NodeId> = roots.iter().cloned().collect();

    let mut depth = 0;
    while !frontier.is_empty() {
        if let Some(max) = max_depth {
            if depth >= max {
                break;
            }
        }

        let mut expanded: Vec<ExpandedImportNode> = frontier
            .par_iter()
            .map(|node| {
                let Some(path) = node.as_file() else {
                    return ExpandedImportNode {
                        node: node.clone(),
                        neighbors: Vec::new(),
                        collected: None,
                    };
                };
                if !graph_files.is_visible(path) || !is_indexable(path) {
                    return ExpandedImportNode {
                        node: node.clone(),
                        neighbors: Vec::new(),
                        collected: None,
                    };
                }
                let (neighbors, collected) =
                    import_neighbors(path, &resolver, workspace, graph_files, allowed, facts);
                ExpandedImportNode {
                    node: node.clone(),
                    neighbors,
                    collected: if facts.retain_collected {
                        collected.map(|facts| (path.to_path_buf(), facts))
                    } else {
                        None
                    },
                }
            })
            .collect();
        // ⚡ Bolt: Use `sort_by_cached_key` instead of `sort_by_key` to avoid repeatedly calling
        // `node_sort_key` (which involves allocation and formatting) during the sort operations.
        expanded.sort_by_cached_key(|expanded| node_sort_key(&expanded.node));

        let next_depth = depth + 1;
        let mut next_frontier = Vec::new();
        for expanded in expanded {
            let ExpandedImportNode {
                node,
                neighbors,
                collected,
            } = expanded;
            if let Some(facts) = collected {
                collected_facts.push(facts);
            }
            for (neighbor, kind) in neighbors {
                if is_symbol_owner_bridge(&node, &neighbor) && !root_nodes.contains(&node) {
                    continue;
                }
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
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

    (result, collected_facts)
}

fn push_route_ref_edge(edges: &mut Vec<Edge>, source: &Path, target: &Path) {
    edges.push((
        NodeId::File(source.to_path_buf()),
        NodeId::File(target.to_path_buf()),
        EdgeKind::RouteRef,
    ));
}
