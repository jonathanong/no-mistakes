pub(crate) fn lazy_import_deps_of_with_files_facts_workspace_and_resolution_cache(
    input: LazyImportBuild<'_>,
) -> (Vec<NodeEntry>, Vec<(PathBuf, TsFileFacts)>) {
    let session = crate::codebase::analysis_session::AnalysisSession::new(
        crate::diagnostics::current(),
    );
    lazy_import_deps_of_with_files_facts_workspace_resolution_cache_and_session(input, &session)
}

pub(crate) fn lazy_import_deps_of_with_files_facts_workspace_resolution_cache_and_session(
    input: LazyImportBuild<'_>,
    session: &crate::codebase::analysis_session::AnalysisSession,
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
    let resolver = ImportResolver::new_observed(tsconfig, session.observer().cloned())
        .with_visible(&graph_files.visible);
    let resolver = match import_resolution_cache {
        Some(cache) => resolver.with_shared_cache(cache),
        None => resolver,
    };
    let fact_plan = facts.collect_plan;
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
        if crate::invocation::check_timeout().is_err() {
            break;
        }
        if max_depth.is_some_and(|max| depth >= max) {
            break;
        }

        let mut expanded: Vec<ExpandedImportNode> = frontier
            .par_iter()
            .map(|node| {
                crate::invocation::check_timeout().ok().map(|()| {
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
                let (neighbors, collected) = import_neighbors(
                    path,
                    &resolver,
                    workspace,
                    graph_files,
                    allowed,
                    facts,
                    session,
                );
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
            })
            .while_some()
            .collect();
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
                } else if let Some(&idx) = result_idx.get(&neighbor) {
                    add_via_kind(&mut result[idx], kind);
                }
            }
        }
        frontier = next_frontier;
        depth = next_depth;
    }

    session.record_work("traversal.lazy_nodes", result.len() as u64);
    (
        result,
        TsFactMap::from_iter_with_plan(collected_facts, fact_plan)
            .into_iter()
            .collect(),
    )
}
