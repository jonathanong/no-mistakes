fn resolve_symbol_dependents(
    root: &Path,
    entrypoints: &[Entrypoint],
    depth: Option<usize>,
    allowed: Option<&std::collections::HashSet<EdgeKind>>,
    graph: &graph::DepGraph,
    symbol_index: &graph::SymbolIndex,
) -> Vec<graph::NodeEntry> {
    let mut all_entries: HashMap<NodeId, graph::NodeEntry> = HashMap::new();
    let plain_roots: Vec<_> = entrypoints
        .iter()
        .filter(|ep| ep.symbol.is_none() || matches!(ep.node, graph::NodeId::Module(_)))
        .map(|ep| ep.node.clone())
        .collect();
    if !plain_roots.is_empty() {
        let entries = graph.dependents_of(&plain_roots, depth, allowed);
        merge_node_entries(&mut all_entries, entries);
    }
    for ep in entrypoints {
        if let Some(sym) = &ep.symbol {
            if matches!(ep.node, graph::NodeId::Module(_)) {
                continue;
            }
            let entries = graph.dependents_of_symbol(&ep.file, sym, depth, allowed, symbol_index);
            merge_node_entries(&mut all_entries, entries);
        }
    }
    let mut entries: Vec<_> = all_entries.into_values().collect();
    sort_node_entries(&mut entries, root);
    entries
}

fn build_dependents_graph(
    ctx: &TraversalCtx<'_>,
    symbol_facts: Option<&crate::codebase::ts_source::facts::TsFactMap>,
) -> graph::DepGraph {
    match symbol_facts {
        Some(facts) => graph::DepGraph::build_with_plan_files_and_facts(
            ctx.root,
            ctx.tsconfig,
            ctx.build_plan,
            ctx.graph_files,
            Some(facts as &dyn graph::TsFactLookup),
        ),
        None => graph::DepGraph::build_with_plan_and_files(
            ctx.root,
            ctx.tsconfig,
            ctx.build_plan,
            ctx.graph_files,
        ),
    }
}

fn write_entries(
    format: Format,
    root_strs: &[String],
    entries: &[graph::NodeEntry],
    root: &Path,
    out: &mut dyn Write,
) -> Result<()> {
    match format {
        Format::Json => output::write_json(root_strs, entries, root, out),
        Format::Md => output::write_md(root_strs, entries, root, out),
        Format::Yml => output::write_yml(root_strs, entries, root, out),
        Format::Paths => output::write_paths(entries, root, out),
        Format::Human => output::write_human(root_strs, entries, root, out),
    }
}

fn resolve_format(json: bool, format: Option<Format>, stdout_is_terminal: bool) -> Format {
    if json {
        Format::Json
    } else if let Some(format) = format {
        format
    } else if stdout_is_terminal {
        Format::Human
    } else {
        Format::Json
    }
}

fn sort_node_entries(entries: &mut [graph::NodeEntry], root: &Path) {
    entries.sort_by_cached_key(|entry| (entry.depth, entry.node.display_name(root)));
}

fn merge_node_entries(
    merged: &mut HashMap<NodeId, graph::NodeEntry>,
    entries: Vec<graph::NodeEntry>,
) {
    for entry in entries {
        if let Some(existing) = merged.get_mut(&entry.node) {
            existing.depth = existing.depth.min(entry.depth);
            existing.via.extend(entry.via.iter().copied());
            existing.via.sort_by_key(|kind| *kind as u8);
            existing.via.dedup();
        } else {
            merged.insert(entry.node.clone(), entry);
        }
    }
}
