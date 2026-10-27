/// A syntactic selector projection for policies that count declared imports,
/// including unused ones. It shares the canonical graph, facts, and resolver,
/// while remaining distinct from legacy symbol execution reachability.
fn collect_symbol_import_selectors(
    input: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> Vec<Edge> {
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
    input
        .graph_files
        .indexable()
        .par_iter()
        .flat_map(|path| {
            let Some(file_facts) = facts.get_ts_facts(path) else {
                return Vec::new();
            };
            let Some(symbols) = file_facts.symbols.as_ref() else {
                return Vec::new();
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
        .collect()
}

fn selector_target(inputs: &ExportEdgeInputs<'_>, source: &str) -> Option<PathBuf> {
    let classification =
        inputs
            .resolver
            .classify_import(source, inputs.path, inputs.workspace, inputs.graph_files);
    // `preferred_path` keeps an invisible TS target and never consults the
    // workspace exports entry. Match resolve-check: a visible resolver path,
    // otherwise the prepared workspace source unless a paths alias owns it.
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

/// A paths rewrite that drops the specifier is an alias, not a baseUrl join.
fn paths_alias_owns(inputs: &ExportEdgeInputs<'_>, source: &str) -> bool {
    if source.starts_with(['.', '/', '#']) {
        return false;
    }
    inputs
        .resolver
        .resolution_candidates(source, inputs.path)
        .iter()
        .any(|candidate| !candidate_contains_specifier(candidate, source))
}

fn candidate_contains_specifier(candidate: &Path, specifier: &str) -> bool {
    let text = candidate.to_string_lossy();
    let needle = format!("/{specifier}");
    text.match_indices(&needle).any(|(index, _)| {
        matches!(
            text[index + needle.len()..].chars().next(),
            None | Some('.' | '/')
        )
    })
}

fn selector_unknown(inputs: &ExportEdgeInputs<'_>, source: &str, edges: &mut Vec<Edge>) {
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
    edges: &mut Vec<Edge>,
) {
    // A known file with no concrete export (including ambiguous/default stars)
    // cannot be attached to an owning export, so preserve a global diagnostic.
    edges.push((
        NodeId::file_in(inputs.interner, inputs.path),
        NodeId::module(format!("{}#{name}", file.display())),
        EdgeKind::SymbolImportUnknown,
    ));
}
