fn import_neighbors(
    path: &Path,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
    graph_files: &GraphFiles,
    allowed: Option<&HashSet<EdgeKind>>,
    prepared: Option<&dyn TsFactLookup>,
) -> Vec<(NodeId, EdgeKind)> {
    let owned_facts;
    let file_facts = if let Some(facts) = prepared.and_then(|facts| facts.get_ts_facts(path)) {
        facts
    } else {
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(_) => return Vec::new(),
        };
        let facts = crate::ast::with_program(path, &source, |program, _| {
            crate::codebase::dependencies::extract::extract_import_facts_from_program(program)
        })
        .unwrap_or_default();
        owned_facts = crate::codebase::ts_source::facts::TsFileFacts {
            imports: facts.imports,
            function_calls: facts.function_calls,
            exported_functions: facts.exported_functions,
            unknown_callers: facts.unknown_callers,
            has_unknown_top_level_call: facts.has_unknown_top_level_call,
            ..Default::default()
        };
        &owned_facts
    };
    let reachable = reachable_function_scopes(file_facts);
    let mut neighbors: Vec<(NodeId, EdgeKind)> = file_facts
        .imports
        .iter()
        .filter(|imp| import_is_reachable(imp, file_facts, &reachable))
        .filter_map(|imp| {
            let kind = edge_kind_for_import(imp);
            if let Some(target) = resolver
                .resolve(&imp.specifier, path)
                .filter(|target| graph_files.is_visible(target) && is_indexable(target))
            {
                return Some((NodeId::File(target), kind));
            }
            if workspace
                .resolve_specifier_from_file_visible(
                    &imp.specifier,
                    path,
                    graph_files.visible(),
                )
                .is_some()
            {
                return None;
            }
            if workspace.recognizes_specifier_from(&imp.specifier, path) {
                return None;
            }
            bare_module_node(&imp.specifier).map(|module| (module, kind))
        })
        .filter(|(_, kind)| allowed.is_none_or(|a| a.contains(kind)))
        .collect();
    // ⚡ Bolt: Use `sort_by_cached_key` instead of `sort_by_key` to avoid repeatedly calling
    // `node_sort_key` (which involves allocation and formatting) during the sort operations.
    neighbors.sort_by_cached_key(|(node, kind)| (node_sort_key(node), *kind as u8));
    neighbors
}
