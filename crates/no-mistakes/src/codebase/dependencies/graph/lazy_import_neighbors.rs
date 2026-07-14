fn import_neighbors(
    path: &Path,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
    graph_files: &GraphFiles,
    allowed: Option<&HashSet<EdgeKind>>,
    fact_source: LazyImportFacts<'_>,
) -> (Vec<(NodeId, EdgeKind)>, Option<TsFileFacts>) {
    if let Some(facts) = fact_source
        .prepared
        .and_then(|facts| facts.get_ts_facts(path))
    {
        return (
            import_neighbors_from_facts(
                path,
                facts,
                resolver,
                workspace,
                graph_files,
                allowed,
            ),
            None,
        );
    }

    let facts = {
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                return (
                    Vec::new(),
                    Some(TsFileFacts {
                        parse_error: Some(format!("failed to read {}: {error}", path.display())),
                        ..TsFileFacts::default()
                    }),
                );
            }
        };
        match crate::ast::with_program(path, &source, |program, source| {
            crate::codebase::ts_source::facts::collect_file_facts_from_program(
                path,
                fact_source.collect_plan,
                fact_source.context,
                source,
                program,
                None,
            )
        }) {
            Ok(facts) => facts,
            Err(error) => TsFileFacts {
                parse_error: Some(error.to_string()),
                ..TsFileFacts::default()
            },
        }
    };

    let neighbors = import_neighbors_from_facts(
        path,
        &facts,
        resolver,
        workspace,
        graph_files,
        allowed,
    );
    (neighbors, Some(facts))
}

fn import_neighbors_from_facts(
    path: &Path,
    file_facts: &TsFileFacts,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
    graph_files: &GraphFiles,
    allowed: Option<&HashSet<EdgeKind>>,
) -> Vec<(NodeId, EdgeKind)> {
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
