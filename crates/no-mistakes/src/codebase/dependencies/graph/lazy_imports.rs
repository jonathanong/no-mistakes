pub(crate) fn lazy_import_deps_of_with_files_facts_workspace_and_resolution_cache(
    input: LazyImportBuild<'_>,
) -> (Vec<NodeEntry>, Vec<(PathBuf, TsFileFacts)>) {
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    lazy_import_deps_of_with_files_facts_workspace_resolution_cache_and_session(input, &session)
}

pub(crate) fn lazy_import_deps_of_with_files_facts_workspace_resolution_cache_and_session(
    input: LazyImportBuild<'_>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> (Vec<NodeEntry>, Vec<(PathBuf, TsFileFacts)>) {
    let LazyImportBuild {
        roots,
        tsconfig,
        tsconfig_catalog,
        max_depth,
        graph_files,
        allowed,
        facts,
        workspace,
        import_resolution_cache,
    } = input;
    let resolver = crate::codebase::ts_resolver::ProjectImportResolver::new(
        tsconfig,
        tsconfig_catalog,
        graph_files,
        import_resolution_cache,
        session,
    );
    let fact_plan = facts.collect_plan;
    // Intern owns each NodeId once. Clone a neighbor only into that map, then
    // move it onto the next frontier; rebuild NodeEntry results at the end.
    let mut intern: FxHashMap<NodeId, LazyVisit> = fx_map();
    let mut frontier: Vec<NodeId> = Vec::new();
    let mut collected_facts = Vec::new();
    let mut emit_order = 0usize;

    for root in roots {
        if intern.contains_key(root) {
            continue;
        }
        intern.insert(
            root.clone(),
            LazyVisit {
                result_order: None,
                depth: 0,
                via: Vec::new(),
            },
        );
        frontier.push(root.clone());
    }
    let root_nodes: FxHashSet<NodeId> = roots.iter().cloned().collect();

    let mut depth = 0;
    while !frontier.is_empty() && crate::invocation::check_timeout().is_ok() {
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
                    if !graph_files.contains_visible(path) || !is_indexable(path) {
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
        expanded.sort_by(|left, right| cmp_node_sort_keys(&left.node, &right.node));

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
                if let Some(visit) = intern.get_mut(&neighbor) {
                    if visit.result_order.is_some() {
                        add_via_kind_to(&mut visit.via, kind);
                    }
                } else {
                    intern.insert(
                        neighbor.clone(),
                        LazyVisit {
                            result_order: Some(emit_order),
                            depth: next_depth,
                            via: vec![kind],
                        },
                    );
                    emit_order += 1;
                    next_frontier.push(neighbor);
                }
            }
        }
        frontier = next_frontier;
        depth = next_depth;
    }

    let mut ordered: Vec<(usize, NodeEntry)> = intern
        .into_iter()
        .filter_map(|(node, visit)| {
            Some((
                visit.result_order?,
                NodeEntry {
                    node,
                    depth: visit.depth,
                    via: visit.via,
                },
            ))
        })
        .collect();
    ordered.sort_unstable_by_key(|(order, _)| *order);
    let result: Vec<NodeEntry> = ordered.into_iter().map(|(_, entry)| entry).collect();

    session.record_work("traversal.lazy_nodes", result.len() as u64);
    (
        result,
        TsFactMap::from_iter_with_plan(collected_facts, fact_plan)
            .into_iter()
            .collect(),
    )
}
