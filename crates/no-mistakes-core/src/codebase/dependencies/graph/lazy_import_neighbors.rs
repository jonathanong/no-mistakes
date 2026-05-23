fn import_neighbors(
    path: &Path,
    resolver: &ImportResolver<'_>,
    ts_ex: &ImportExtractor,
    tsx_ex: &ImportExtractor,
    graph_files: &GraphFiles,
    allowed: Option<&HashSet<EdgeKind>>,
) -> Vec<(NodeId, EdgeKind)> {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(_) => return Vec::new(),
    };
    let _extractor = if is_tsx_file(path) { tsx_ex } else { ts_ex };
    let facts = crate::ast::with_program(path, &source, |program, _| {
        crate::codebase::dependencies::extract::extract_import_facts_from_program(program)
    })
    .unwrap_or_default();
    let reachable = reachable_function_scopes(&crate::codebase::ts_source::facts::TsFileFacts {
        imports: facts.imports.clone(),
        function_calls: facts.function_calls.clone(),
        exported_functions: facts.exported_functions.clone(),
        has_unknown_top_level_call: facts.has_unknown_top_level_call,
        ..Default::default()
    });
    let file_facts = crate::codebase::ts_source::facts::TsFileFacts {
        imports: facts.imports,
        function_calls: facts.function_calls,
        exported_functions: facts.exported_functions,
        has_unknown_top_level_call: facts.has_unknown_top_level_call,
        ..Default::default()
    };
    let mut neighbors: Vec<(NodeId, EdgeKind)> = file_facts
        .imports
        .iter()
        .filter(|imp| import_is_reachable(imp, &file_facts, &reachable))
        .filter_map(|imp| {
            let kind = edge_kind_for_import(imp);
            if let Some(target) = resolver
                .resolve(&imp.specifier, path)
                .filter(|target| graph_files.is_visible(target) && is_indexable(target))
            {
                return Some((NodeId::File(target), kind));
            }
            bare_module_node(&imp.specifier).map(|module| (module, kind))
        })
        .filter(|(_, kind)| allowed.is_none_or(|a| a.contains(kind)))
        .collect();
    neighbors.sort_by_key(|(node, kind)| (node_sort_key(node), *kind as u8));
    neighbors
}
