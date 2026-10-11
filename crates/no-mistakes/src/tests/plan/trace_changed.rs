use super::*;

pub(super) struct TraceChangedContext<'a> {
    pub args: &'a PlanArgs,
    pub graph: &'a DepGraph,
    pub test_filter: &'a TestFileFilter,
    pub root: &'a Path,
    pub changed_files: &'a [PathBuf],
}

pub(super) struct TraceChangedState<'a> {
    pub selected_map: &'a mut HashMap<PathBuf, SelectedTest>,
    pub warnings: &'a mut Vec<Warning>,
    pub warnings_seen: &'a mut HashSet<WarningKey>,
}

pub(super) fn trace_changed_files(context: TraceChangedContext<'_>, state: TraceChangedState<'_>) {
    let TraceChangedContext {
        args,
        graph,
        test_filter,
        root,
        changed_files,
    } = context;
    let TraceChangedState {
        selected_map,
        warnings,
        warnings_seen,
    } = state;
    // 4. Trace each changed file
    for changed in changed_files {
        let basename = changed.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if no_mistakes::codebase::lockfile::detect_manager(basename).is_some()
            || no_mistakes::codebase::lockfile::is_binary_lockfile(basename)
        {
            continue;
        }

        let rel_changed = relative_path(root, changed);

        // Dynamic resource calls are intentionally edge-less. A changed
        // consumer remains relevant even when no static path reaches a test.
        push_resource_diagnostics(graph, root, changed, warnings, warnings_seen);

        // If the changed file is a test file itself, select it directly
        if test_filter.is_match(root, changed) {
            let entry = selected_map
                .entry(changed.clone())
                .or_insert_with(|| SelectedTest {
                    test_file: rel_changed.clone(),
                    confidence: Confidence::High,
                    targets: Vec::new(),
                    reasons: Vec::new(),
                });
            entry.confidence = Confidence::High;
            let reason = ImpactReason {
                changed_file: rel_changed.clone(),
                path: vec![rel_changed.clone()],
                via: vec!["self".to_string()],
                via_details: Vec::new(),
            };
            if !entry.reasons.contains(&reason) {
                entry.reasons.push(reason);
            }
            continue;
        }

        // Otherwise, run BFS path finder in reverse direction
        let start_nodes = changed_start_nodes(graph, changed, args.include_symbols);

        for start_node in start_nodes {
            let (reachable_tests, path_parents) =
                bfs_path_find(graph, &start_node, test_filter, root);

            for (test_node, edge_path) in reachable_tests {
                let test_path = match &test_node {
                    NodeId::File(p) => p.to_path_buf(),
                    _ => continue,
                };
                let rel_test = relative_path(root, &test_path);

                // Compute confidence of the path
                let path_conf = path_confidence(&edge_path);

                // Reconstruct path node chain and collect warnings in a single pass
                let mut node_chain = Vec::new();
                let mut reverse_details = Vec::new();
                let mut curr = test_node.clone();
                node_chain.push(slash_node_name(&curr, root));

                while let Some((parent, kind)) = path_parents.get(&curr) {
                    if let Some(file) = curr.as_file() {
                        push_resource_diagnostics(graph, root, file, warnings, warnings_seen);
                    }
                    node_chain.push(slash_node_name(parent, root));
                    reverse_details.push(resource_edge_detail(graph, &curr, parent, *kind, root));

                    match kind {
                        EdgeKind::DynamicImport | EdgeKind::ConditionalDynamicImport => {
                            let warn = Warning {
                                r#type: "dynamic-import".to_string(),
                                message: format!(
                                    "Dynamic import in `{}` might not be fully resolved.",
                                    slash_node_name(&curr, root)
                                ),
                                file: slash_node_name(&curr, root),
                                line: None,
                            };
                            if warnings_seen.insert(warning_key(&warn)) {
                                warnings.push(warn);
                            }
                        }
                        EdgeKind::HttpCall => {
                            let warn = Warning {
                                r#type: "http-call".to_string(),
                                message: format!(
                                    "Dynamic HTTP call in `{}` to backend `{}`.",
                                    slash_node_name(&curr, root),
                                    slash_node_name(parent, root)
                                ),
                                file: slash_node_name(&curr, root),
                                line: None,
                            };
                            if warnings_seen.insert(warning_key(&warn)) {
                                warnings.push(warn);
                            }
                        }
                        EdgeKind::ProcessSpawn => {
                            let warn = Warning {
                                r#type: "process-spawn".to_string(),
                                message: format!(
                                    "Process spawned in `{}`.",
                                    slash_node_name(&curr, root)
                                ),
                                file: slash_node_name(&curr, root),
                                line: None,
                            };
                            if warnings_seen.insert(warning_key(&warn)) {
                                warnings.push(warn);
                            }
                        }
                        _ => {}
                    }
                    curr = parent.clone();
                }
                if let Some(file) = curr.as_file() {
                    push_resource_diagnostics(graph, root, file, warnings, warnings_seen);
                }
                node_chain.reverse();
                reverse_details.reverse();

                let via_strings: Vec<String> = edge_path
                    .iter()
                    .map(|k| impact_reason_label(*k).to_string())
                    .collect();

                let reason = ImpactReason {
                    changed_file: rel_changed.clone(),
                    path: node_chain,
                    via: via_strings,
                    via_details: reverse_details,
                };

                let entry = selected_map
                    .entry(test_path)
                    .or_insert_with(|| SelectedTest {
                        test_file: rel_test.clone(),
                        confidence: path_conf,
                        targets: Vec::new(),
                        reasons: Vec::new(),
                    });

                // Update confidence to the highest among paths
                if path_conf > entry.confidence {
                    entry.confidence = path_conf;
                }

                if !entry.reasons.contains(&reason) {
                    entry.reasons.push(reason);
                }
            }
        }
    }
}
