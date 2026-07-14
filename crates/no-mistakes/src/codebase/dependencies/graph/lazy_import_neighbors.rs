fn import_neighbors(
    path: &Path,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    graph_files: &GraphFiles,
    allowed: Option<&HashSet<EdgeKind>>,
    fact_source: LazyImportFacts<'_>,
    session: &crate::codebase::analysis_session::AnalysisSession,
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
        let source_result = match fact_source.sources {
            Some(sources) => sources
                .read_path(path)
                .map_err(|error| error.to_string()),
            None => session
                .read_source(path)
                .map_err(|error| error.to_string()),
        };
        let source = match source_result {
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
        match session.with_program(path, &source, |program, source| {
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
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
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
            let classification = resolver.classify_import(
                &imp.specifier,
                path,
                workspace,
                graph_files.visible(),
            );
            if let Some(target) = classification.preferred_path() {
                return (graph_files.is_visible(target) && is_indexable(target))
                    .then(|| (NodeId::File(target.to_path_buf()), kind));
            }
            if classification.is_unresolved_external() {
                return bare_module_node(&imp.specifier).map(|module| (module, kind));
            }
            None
        })
        .filter(|(_, kind)| allowed.is_none_or(|a| a.contains(kind)))
        .collect();
    // ⚡ Bolt: Use `sort_by_cached_key` instead of `sort_by_key` to avoid repeatedly calling
    // `node_sort_key` (which involves allocation and formatting) during the sort operations.
    neighbors.sort_by_cached_key(|(node, kind)| (node_sort_key(node), *kind as u8));
    neighbors
}
