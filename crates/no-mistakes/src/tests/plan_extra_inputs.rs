pub(crate) fn trace_deleted_files(
    deleted_files: &[PathBuf],
    graph: &DepGraph,
    test_filter: &TestFileFilter,
    root: &Path,
    selected_map: &mut HashMap<PathBuf, SelectedTest>,
    warnings: &mut Vec<Warning>,
    warnings_seen: &mut HashSet<WarningKey>,
) {
    for deleted in deleted_files {
        let start_node = NodeId::File(deleted.clone());
        let rel_deleted = relative_path(root, deleted);
        if let Some(neighbors) = graph.dependents_of_node(&start_node) {
            for (neighbor, _kind) in neighbors {
                let NodeId::File(neighbor_path) = neighbor else {
                    continue;
                };
                if test_filter.is_match(root, neighbor_path) {
                    add_deleted_direct(
                        neighbor_path,
                        &rel_deleted,
                        root,
                        selected_map,
                    );
                } else {
                    add_deleted_transitive(
                        neighbor_path,
                        &rel_deleted,
                        graph,
                        test_filter,
                        root,
                        selected_map,
                    );
                }
            }
        }
        let warn = Warning {
            r#type: "deleted-file".to_string(),
            message: format!("File `{}` was deleted.", rel_deleted),
            file: rel_deleted,
            line: None,
        };
        if warnings_seen.insert(warning_key(&warn)) {
            warnings.push(warn);
        }
    }
}

fn owner_widened_neighbor_allowed(
    root: &Path,
    test_filter: &TestFileFilter,
    graph: &DepGraph,
    neighbor: &NodeId,
    neighbors: &[(NodeId, EdgeKind)],
) -> bool {
    let NodeId::File(path) = neighbor else {
        return false;
    };
    test_filter.is_match(root, path)
        || (!has_symbol_neighbor_for_file(path, neighbors) && !file_has_owned_symbol(graph, path))
}

fn has_symbol_neighbor_for_file(path: &Path, neighbors: &[(NodeId, EdgeKind)]) -> bool {
    neighbors.iter().any(
        |(candidate, _)| matches!(candidate, NodeId::Symbol { file, .. } if file.as_path() == path),
    )
}

fn file_has_owned_symbol(graph: &DepGraph, path: &Path) -> bool {
    let file = path.to_path_buf();
    graph
        .dependencies_of_node(&NodeId::File(file.clone()))
        .is_some_and(|deps| {
            deps.iter().any(|(candidate, _)| {
                matches!(candidate, NodeId::Symbol { file: symbol_file, .. } if *symbol_file == file)
            })
        })
}

fn add_deleted_direct(
    neighbor_path: &Path,
    rel_deleted: &str,
    root: &Path,
    selected_map: &mut HashMap<PathBuf, SelectedTest>,
) {
    let rel_test = relative_path(root, neighbor_path);
    let reason = ImpactReason {
        changed_file: rel_deleted.to_string(),
        path: vec![rel_deleted.to_string(), rel_test.clone()],
        via: vec!["deleted-dependency".to_string()],
        via_details: Vec::new(),
    };
    let entry = selected_map
        .entry(neighbor_path.to_path_buf())
        .or_insert_with(|| SelectedTest {
            test_file: rel_test,
            confidence: Confidence::High,
            targets: Vec::new(),
            reasons: Vec::new(),
        });
    if !entry.reasons.contains(&reason) {
        entry.reasons.push(reason);
    }
}

fn add_deleted_transitive(
    neighbor_path: &Path,
    rel_deleted: &str,
    graph: &DepGraph,
    test_filter: &TestFileFilter,
    root: &Path,
    selected_map: &mut HashMap<PathBuf, SelectedTest>,
) {
    let (reachable, parents) = bfs_path_find(
        graph,
        &NodeId::File(neighbor_path.to_path_buf()),
        test_filter,
        root,
    );
    for (test_node, edge_path) in reachable {
        let NodeId::File(test_path) = &test_node else {
            continue;
        };
        let rel_test = relative_path(root, test_path);
        let path_conf = path_confidence(&edge_path);
        let mut node_chain = vec![slash_node_name(&test_node, root)];
        let mut curr = test_node.clone();
        while let Some((parent, _)) = parents.get(&curr) {
            node_chain.push(slash_node_name(parent, root));
            curr = parent.clone();
        }
        node_chain.push(rel_deleted.to_string());
        node_chain.reverse();
        let via_strings: Vec<String> = std::iter::once("deleted-dependency".to_string())
            .chain(
                edge_path
                    .iter()
                    .map(|k| impact_reason_label(*k).to_string()),
            )
            .collect();
        let reason = ImpactReason {
            changed_file: rel_deleted.to_string(),
            path: node_chain,
            via: via_strings,
            via_details: Vec::new(),
        };
        let entry = selected_map
            .entry(test_path.clone())
            .or_insert_with(|| SelectedTest {
                test_file: rel_test,
                confidence: path_conf,
                targets: Vec::new(),
                reasons: Vec::new(),
            });
        if path_conf > entry.confidence {
            entry.confidence = path_conf;
        }
        if !entry.reasons.contains(&reason) {
            entry.reasons.push(reason);
        }
    }
}

pub(crate) fn trace_entrypoints(
    entrypoints: &[String],
    entrypoint_symbols: &[Option<String>],
    graph: &DepGraph,
    test_filter: &TestFileFilter,
    root: &Path,
    selected_map: &mut HashMap<PathBuf, SelectedTest>,
    include_symbols: bool,
) -> Result<()> {
    for (index, raw) in entrypoints.iter().enumerate() {
        let structured_symbol = entrypoint_symbols.get(index).cloned().flatten();
        let structured_entrypoint = structured_symbol.is_some();
        let (raw_file, parsed_symbol) = if structured_entrypoint {
            (PathBuf::from(raw), None)
        } else {
            no_mistakes::codebase::dependencies::parse_entrypoint(raw)
        };
        let symbol = structured_symbol
            .filter(|symbol| !symbol.is_empty())
            .or(parsed_symbol);
        if symbol.is_some() && !include_symbols {
            anyhow::bail!(
                "Entrypoint `{raw}` uses `#symbol`; pass --symbols to enable symbol traversal."
            );
        }
        let file = if raw_file.is_absolute() {
            raw_file
        } else {
            root.join(&raw_file)
        };
        let normalized = no_mistakes::codebase::ts_resolver::normalize_path(&file);
        let display_start_node = symbol.as_ref().filter(|_| include_symbols).map_or_else(
            || NodeId::File(normalized.clone()),
            |symbol| NodeId::Symbol {
                file: normalized.clone(),
                symbol: symbol.clone(),
            },
        );
        let start_nodes = symbol_aware_start_nodes(graph, &normalized, symbol.as_ref(), include_symbols);
        let rel_changed = symbol
            .as_ref()
            .filter(|_| include_symbols)
            .map_or_else(|| relative_path(root, &normalized), |symbol| {
                format!("{}#{}", relative_path(root, &normalized), symbol)
            });

        if test_filter.is_match(root, &normalized) {
            let rel_test = relative_path(root, &normalized);
            let entry = selected_map
                .entry(normalized.clone())
                .or_insert_with(|| SelectedTest {
                    test_file: rel_test,
                    confidence: Confidence::High,
                    targets: Vec::new(),
                    reasons: Vec::new(),
                });
            let reason = ImpactReason {
                changed_file: rel_changed,
                path: vec![slash_node_name(&display_start_node, root)],
                via: vec!["self".to_string()],
                via_details: Vec::new(),
            };
            if !entry.reasons.contains(&reason) {
                entry.reasons.push(reason);
            }
            continue;
        }

        for start_node in start_nodes {
            let (reachable_tests, path_parents) =
                bfs_path_find(graph, &start_node, test_filter, root);

            for (test_node, edge_path) in reachable_tests {
                let test_path = match &test_node {
                    NodeId::File(p) => p.clone(),
                    _ => continue,
                };
                let rel_test = relative_path(root, &test_path);
                let path_conf = path_confidence(&edge_path);
                let mut node_chain = Vec::new();
                let mut curr = test_node.clone();
                node_chain.push(slash_node_name(&curr, root));
                while let Some((parent, _)) = path_parents.get(&curr) {
                    node_chain.push(slash_node_name(parent, root));
                    curr = parent.clone();
                }
                node_chain.reverse();
                let via_strings: Vec<String> = edge_path
                    .iter()
                    .map(|k| impact_reason_label(*k).to_string())
                    .collect();
                let reason = ImpactReason {
                    changed_file: rel_changed.clone(),
                    path: node_chain,
                    via: via_strings,
                    via_details: Vec::new(),
                };
                let entry = selected_map
                    .entry(test_path)
                    .or_insert_with(|| SelectedTest {
                        test_file: rel_test,
                        confidence: path_conf,
                        targets: Vec::new(),
                        reasons: Vec::new(),
                    });
                if path_conf > entry.confidence {
                    entry.confidence = path_conf;
                }
                if !entry.reasons.contains(&reason) {
                    entry.reasons.push(reason);
                }
            }
        }
    }
    Ok(())
}
