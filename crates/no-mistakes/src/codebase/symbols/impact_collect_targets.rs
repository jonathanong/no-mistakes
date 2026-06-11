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
        EdgeKind::TestOf,
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
    for node in export_nodes {
        match node {
            NodeId::Symbol { file, symbol } => {
                target_symbols
                    .entry(file.clone())
                    .or_default()
                    .insert(symbol.clone());
            }
            NodeId::File(file) => {
                target_symbols.entry(file.clone()).or_default();
            }
            NodeId::Module(_) | NodeId::QueueJob { .. } => {}
        }
    }
    target_symbols
}

fn suggested_test_entries(
    graph: &DepGraph,
    entries: &[NodeEntry],
    production_extra_callers: &[CallerEntry],
    root: &Path,
) -> Vec<NodeEntry> {
    let mut suggested_entries = entries.to_vec();
    let test_edges = HashSet::from([EdgeKind::TestOf]);
    for caller in production_extra_callers {
        let node = NodeId::File(root.join(&caller.file));
        suggested_entries.extend(graph.dependents_of(&[node], None, Some(&test_edges)));
    }
    suggested_entries
}
