fn signature_impact_graph_plan() -> GraphBuildPlan {
    let mut plan = GraphBuildPlan::imports_and_workspace();
    plan.tests = true;
    plan.with_symbols(true)
}

fn signature_impact_edges() -> HashSet<EdgeKind> {
    HashSet::from([
        EdgeKind::Import,
        EdgeKind::TypeImport,
        EdgeKind::DynamicImport,
        EdgeKind::Require,
        EdgeKind::WorkspaceImport,
    ])
}

fn signature_target_symbols(
    target_file: &Path,
    target_symbol: &str,
    export_nodes: &BTreeSet<NodeId>,
) -> BTreeMap<PathBuf, BTreeSet<String>> {
    let mut target_symbols = BTreeMap::from([(
        target_file.to_path_buf(),
        BTreeSet::from([target_symbol.to_string()]),
    )]);
    let mut changed = true;
    while changed {
        changed = false;
        let known_symbols: BTreeSet<String> =
            target_symbols.values().flatten().cloned().collect();
        for node in export_nodes {
            match node {
                NodeId::Symbol { file, symbol } => {
                    let symbol_name = known_symbols
                        .iter()
                        .filter_map(|candidate| {
                            namespace_reexport_target_symbol(file, symbol, candidate)
                        })
                        .max_by_key(|candidate| candidate.matches('.').count())
                        .or_else(|| {
                            (!is_namespace_reexport_symbol(file, symbol)).then(|| symbol.clone())
                        });
                    if let Some(symbol_name) = symbol_name {
                        if target_symbols
                            .entry(file.clone())
                            .or_default()
                            .insert(symbol_name)
                        {
                            changed = true;
                        }
                    }
                }
                NodeId::File(file) => {
                    target_symbols.entry(file.clone()).or_default();
                }
                NodeId::Module(_) | NodeId::QueueJob { .. } => {}
            }
        }
    }
    target_symbols
}

include!("impact_collect_target_helpers.rs");

fn suggested_test_entries(
    graph: &DepGraph,
    entries: &[NodeEntry],
    production_extra_callers: &[CallerEntry],
    root: &Path,
    file_target_symbols: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<NodeEntry> {
    let mut suggested_entries = entries.to_vec();
    let test_edges = HashSet::from([EdgeKind::TestOf]);
    let mut production_files: BTreeSet<PathBuf> = BTreeSet::new();
    for entry in entries {
        if has_file_level_import_edge(&entry.via) {
            let Some(file) = entry.node.as_file() else {
                continue;
            };
            let relative_file = relative_slash_path(root, file);
            if file_target_symbols
                .get(relative_file.as_str())
                .is_some_and(|symbols| file_entry_uses_any_symbol(root, relative_file.as_str(), symbols))
            {
                production_files.insert(file.to_path_buf());
            }
        } else if !entry.via.contains(&EdgeKind::TestOf)
            && matches!(entry.node, NodeId::Symbol { .. })
        {
            if let Some(file) = entry.node.as_file() {
                production_files.insert(file.to_path_buf());
            }
        }
    }
    for caller in production_extra_callers {
        production_files.insert(root.join(&caller.file));
    }
    for file in production_files {
        let node = NodeId::File(file);
        suggested_entries.extend(graph.dependents_of(&[node], None, Some(&test_edges)));
    }
    suggested_entries
}
