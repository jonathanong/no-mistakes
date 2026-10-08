fn import_neighbors(
    path: &Path,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visibility: ImportNeighborVisibility<'_>,
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
                visibility,
                allowed,
                session.interner(),
            ),
            None,
        );
    }
    if let Some(cache) = fact_source.live_cache {
        let facts = cache
            .entry(path.to_path_buf())
            .or_insert_with(|| {
                std::sync::Arc::new(collect_lazy_file_facts(path, fact_source, session))
            })
            .clone();
        return (
            import_neighbors_from_facts(
                path,
                facts.as_ref(),
                resolver,
                workspace,
                visibility,
                allowed,
                session.interner(),
            ),
            fact_source.retain_collected.then(|| facts.as_ref().clone()),
        );
    }

    let facts = collect_lazy_file_facts(path, fact_source, session);
    let neighbors = import_neighbors_from_facts(
        path,
        &facts,
        resolver,
        workspace,
        visibility,
        allowed,
        session.interner(),
    );
    (neighbors, Some(facts))
}

fn collect_lazy_file_facts(
    path: &Path,
    fact_source: LazyImportFacts<'_>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> TsFileFacts {
    let source_result = match fact_source.sources {
        Some(sources) => sources.read_path(path).map_err(|error| error.to_string()),
        None => session.read_source(path).map_err(|error| error.to_string()),
    };
    let source = match source_result {
        Ok(source) => source,
        Err(error) => {
            return TsFileFacts {
                parse_error: Some(format!("failed to read {}: {error}", path.display())),
                ..TsFileFacts::default()
            };
        }
    };
    match session.with_program(path, &source, |program, parsed| {
        crate::codebase::ts_source::facts::collect_file_facts_from_program(
            path,
            fact_source.collect_plan,
            fact_source.context,
            parsed,
            program,
            None,
            if fact_source.collect_plan.source {
                Some(std::sync::Arc::clone(&source))
            } else {
                None
            },
        )
    }) {
        Ok(facts) => facts,
        Err(error) => TsFileFacts {
            parse_error: Some(error.to_string()),
            ..TsFileFacts::default()
        },
    }
}

fn import_neighbors_from_facts(
    path: &Path,
    file_facts: &TsFileFacts,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visibility: ImportNeighborVisibility<'_>,
    allowed: Option<&HashSet<EdgeKind>>,
    interner: &PathInterner,
) -> Vec<(NodeId, EdgeKind)> {
    let reachable = reachable_function_scopes(file_facts);
    let mut neighbors: Vec<(NodeId, EdgeKind)> = file_facts
        .imports
        .iter()
        .flat_map(|imp| {
            let Some(kind) = graph_edge_kind_for_extracted_import(imp, file_facts, &reachable)
            else {
                return vec![];
            };
            let classification =
                resolver.classify_import(&imp.specifier, path, workspace, visibility.lookup());
            let resolver_target = classification
                .resolver_path()
                .and_then(|target| visibility.visible_path(target));
            let workspace_target = classification
                .workspace_path()
                .and_then(|target| visibility.visible_path(target));
            let target = resolver_target.or_else(|| {
                (matches!(imp.kind, ImportKind::Dynamic)
                    || (imp.kind == ImportKind::RequireResolve
                        && classification.resolver_path().is_none()))
                .then(|| workspace_target.clone())
                .flatten()
            });
            let mut neighbors = vec![];
            if let Some(target) = target {
                if is_indexable(&target) || kind == EdgeKind::RequireResolve {
                    neighbors.push((NodeId::file_in(interner, &target), kind));
                }
            } else if classification.is_unresolved_external() {
                if let Some(module) = bare_module_node_in(interner, &imp.specifier) {
                    neighbors.push((module, kind));
                }
            }
            if !matches!(imp.kind, ImportKind::RequireResolve) {
                if let Some(target) = workspace_target {
                    let workspace_kind = if imp.kind == ImportKind::Type {
                        EdgeKind::WorkspaceTypeImport
                    } else {
                        EdgeKind::WorkspaceImport
                    };
                    neighbors.push((NodeId::file_in(interner, &target), workspace_kind));
                }
            }
            neighbors
        })
        .filter(|(_, kind)| allowed.is_none_or(|a| a.contains(kind)))
        .collect();
    neighbors.sort_by(|(left_node, left_kind), (right_node, right_kind)| {
        cmp_node_sort_keys(left_node, right_node)
            .then_with(|| left_kind.sort_key().cmp(&right_kind.sort_key()))
    });
    neighbors
}
