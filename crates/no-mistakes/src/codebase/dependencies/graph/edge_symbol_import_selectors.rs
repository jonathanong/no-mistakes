/// A syntactic selector projection for policies that count declared imports,
/// including unused ones. It shares the canonical graph, facts, and resolver,
/// while remaining distinct from legacy symbol execution reachability.
fn collect_symbol_import_selectors(
    input: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> (Vec<Edge>, SelectorImportSites) {
    let export_edges: FxHashMap<_, _> = input
        .graph_files
        .indexable()
        .par_iter()
        .filter_map(|path| {
            let file_facts = facts.get_ts_facts(path)?;
            let symbols = file_facts.symbols.as_ref()?;
            let inputs = ExportEdgeInputs {
                path,
                symbols,
                facts,
                resolver,
                workspace,
                visible_files: input.graph_files,
                graph_files: input.graph_files,
                interner: &input.interner,
            };
            let mut edges = Vec::new();
            collect_export_edges(inputs, &mut Vec::new(), &mut HashMap::new(), &mut edges);
            Some((path.clone(), edges))
        })
        .collect();
    let exported: FxHashSet<_> = export_edges
        .values()
        .flatten()
        .filter_map(|(from, to, _)| matches!(from, NodeId::File(_)).then_some(to.clone()))
        .collect();
    let chunks: Vec<_> = input
        .graph_files
        .indexable()
        .par_iter()
        .map(|path| {
            let Some(file_facts) = facts.get_ts_facts(path) else {
                return (Vec::new(), fx_map());
            };
            let Some(symbols) = file_facts.symbols.as_ref() else {
                return (Vec::new(), fx_map());
            };
            let inputs = ExportEdgeInputs {
                path,
                symbols,
                facts,
                resolver,
                workspace,
                visible_files: input.graph_files,
                graph_files: input.graph_files,
                interner: &input.interner,
            };
            collect_symbol_import_selectors_file(
                inputs,
                file_facts,
                export_edges
                    .get(path)
                    .map(Vec::as_slice)
                    .unwrap_or_default(),
                &exported,
            )
        })
        .collect();
    let mut edges = Vec::new();
    let mut sites = fx_map();
    for (file_edges, file_sites) in chunks {
        edges.extend(file_edges);
        sites.extend(file_sites);
    }
    (edges, sites)
}

fn selector_target(inputs: &ExportEdgeInputs<'_>, source: &str) -> Option<PathBuf> {
    let classification =
        inputs
            .resolver
            .classify_import(source, inputs.path, inputs.workspace, inputs.graph_files);
    // A visible resolver path wins. Otherwise use the prepared workspace
    // source, unless a tsconfig paths pattern owns the specifier.
    if let Some(path) = visible_selector_path(inputs, classification.resolver_path()) {
        return Some(path);
    }
    let workspace = visible_selector_path(inputs, classification.workspace_path());
    if workspace.is_some()
        && classification.resolver_path().is_none()
        && paths_alias_owns(inputs, source)
    {
        return None;
    }
    workspace
}

fn visible_selector_path(inputs: &ExportEdgeInputs<'_>, path: Option<&Path>) -> Option<PathBuf> {
    path.and_then(|path| inputs.graph_files.visible_path(path))
        .map(Path::to_path_buf)
}

/// A tsconfig `paths` pattern owns the specifier. A `baseUrl` join does not,
/// even when a candidate path happens to contain the specifier text.
fn paths_alias_owns(inputs: &ExportEdgeInputs<'_>, source: &str) -> bool {
    if source.starts_with(['.', '/', '#']) {
        return false;
    }
    inputs.resolver.matches_paths_alias(source, inputs.path)
}

fn selector_unknown(
    inputs: &ExportEdgeInputs<'_>,
    source: &str,
    line: usize,
    reason: &'static str,
    edges: &mut Vec<Edge>,
    sites: &mut SelectorImportSites,
) {
    let target = if let Some(target) = selector_target(inputs, source) {
        // Assets cannot own TS/JS production exports.
        if !is_indexable(&target) {
            return;
        }
        NodeId::file_in(inputs.interner, target)
    } else if source.starts_with('.')
        || source.starts_with('/')
        || inputs
            .workspace
            .recognizes_specifier_from(source, inputs.path)
        || !inputs
            .resolver
            .resolution_candidates(source, inputs.path)
            .is_empty()
    {
        NodeId::module(source)
    } else {
        // External packages are outside this repository's export catalog.
        return;
    };
    push_selector_site(
        inputs,
        target.clone(),
        SelectorImportSite {
            line: line.max(1),
            specifier: source.to_owned(),
            reason,
        },
        sites,
    );
    edges.push((
        NodeId::file_in(inputs.interner, inputs.path),
        target,
        EdgeKind::SymbolImportUnknown,
    ));
}

fn selector_unknown_symbol(
    inputs: &ExportEdgeInputs<'_>,
    file: &Path,
    name: &str,
    site: SelectorImportSite,
    edges: &mut Vec<Edge>,
    sites: &mut SelectorImportSites,
) {
    // A known file with no concrete export (including ambiguous/default stars)
    // cannot be attached to an owning export, so preserve a global diagnostic.
    let target = NodeId::module(format!("{}#{name}", file.display()));
    push_selector_site(inputs, target.clone(), site, sites);
    edges.push((
        NodeId::file_in(inputs.interner, inputs.path),
        target,
        EdgeKind::SymbolImportUnknown,
    ));
}
