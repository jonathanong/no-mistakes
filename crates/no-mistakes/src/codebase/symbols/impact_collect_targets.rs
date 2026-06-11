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
                let symbol_name = namespace_reexport_target_symbol(file, symbol, target_symbol)
                    .unwrap_or_else(|| symbol.clone());
                target_symbols
                    .entry(file.clone())
                    .or_default()
                    .insert(symbol_name);
            }
            NodeId::File(file) => {
                target_symbols.entry(file.clone()).or_default();
            }
            NodeId::Module(_) | NodeId::QueueJob { .. } => {}
        }
    }
    target_symbols
}

fn namespace_reexport_target_symbol(
    file: &Path,
    symbol: &str,
    target_symbol: &str,
) -> Option<String> {
    let source = std::fs::read_to_string(file).ok()?;
    let is_tsx = file
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("tsx") || ext.eq_ignore_ascii_case("jsx"));
    let symbols = extract_symbols(&source, is_tsx).ok()?;
    symbols.exports.iter().find_map(|export| match &export.kind {
        ExportKind::ReExport { imported, .. } if imported == "*" && export.name == symbol => {
            Some(format!("{symbol}.{target_symbol}"))
        }
        _ => None,
    })
}

fn suggested_test_entries(
    graph: &DepGraph,
    entries: &[NodeEntry],
    production_extra_callers: &[CallerEntry],
    root: &Path,
) -> Vec<NodeEntry> {
    let mut suggested_entries = entries.to_vec();
    let test_edges = HashSet::from([EdgeKind::TestOf]);
    let mut production_files: BTreeSet<PathBuf> = entries
        .iter()
        .filter(|entry| entry.via.contains(&EdgeKind::DynamicImport) || entry.via.contains(&EdgeKind::Require))
        .filter_map(|entry| entry.node.as_file().map(Path::to_path_buf))
        .collect();
    for caller in production_extra_callers {
        production_files.insert(root.join(&caller.file));
    }
    for file in production_files {
        let node = NodeId::File(file);
        suggested_entries.extend(graph.dependents_of(&[node], None, Some(&test_edges)));
    }
    suggested_entries
}
