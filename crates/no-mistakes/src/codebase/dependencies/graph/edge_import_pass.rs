fn import_file_pass(
    path: &Path,
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    reachable: &std::collections::HashSet<crate::codebase::dependencies::extract::CallableId>,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    graph_files: &GraphFiles,
    interner: &PathInterner,
) -> (PathBuf, Vec<Edge>, Vec<(String, ImportTarget)>) {
    let mut edges = Vec::new();
    let mut targets = Vec::new();
    for imp in &facts.imports {
        let Some(kind) = graph_edge_kind_for_extracted_import(imp, facts, reachable) else {
            continue;
        };
        let classification = resolver.classify_import(&imp.specifier, path, workspace, graph_files);
        let resolved = resolved_import_path(kind, &classification, imp, graph_files);
        let recorded = match resolved {
            Some(target) => ImportTarget::File(target.to_path_buf()),
            None => ImportTarget::Unresolved,
        };
        targets.push((imp.specifier.clone(), recorded));
        if let Some(edge) = import_edge(interner, path, kind, imp, resolved, &classification) {
            edges.push(edge);
        }
    }
    (path.to_path_buf(), edges, targets)
}

fn resolved_import_path<'a>(
    kind: EdgeKind,
    classification: &crate::codebase::ts_resolver::ImportClassification,
    imp: &ExtractedImport,
    graph_files: &'a GraphFiles,
) -> Option<&'a Path> {
    if kind == EdgeKind::RequireResolve {
        return classification
            .preferred_path()
            .and_then(|target| graph_files.visible_path(target));
    }
    classification
        .resolver_path()
        .and_then(|target| graph_files.visible_path(target))
        // Workspace exports may resolve through an invisible node_modules symlink.
        // Keep the literal dynamic edge kind while using the prepared visible target.
        .or_else(|| {
            matches!(imp.kind, ImportKind::Dynamic)
                .then(|| {
                    classification
                        .workspace_path()
                        .and_then(|target| graph_files.visible_path(target))
                })
                .flatten()
        })
}

fn import_edge(
    interner: &PathInterner,
    path: &Path,
    kind: EdgeKind,
    imp: &ExtractedImport,
    resolved: Option<&Path>,
    classification: &crate::codebase::ts_resolver::ImportClassification,
) -> Option<Edge> {
    if let Some(target) = resolved {
        return (is_indexable(target) || kind == EdgeKind::RequireResolve).then(|| {
            (
                NodeId::file_in(interner, path),
                NodeId::file_in(interner, target),
                kind,
            )
        });
    }
    if classification.is_unresolved_external() {
        return bare_module_node_in(interner, &imp.specifier)
            .map(|module| (NodeId::file_in(interner, path), module, kind));
    }
    None
}
