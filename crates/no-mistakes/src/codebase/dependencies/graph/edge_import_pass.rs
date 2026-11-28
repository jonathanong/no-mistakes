fn import_file_pass(
    path: &Path,
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    reachable: &std::collections::HashSet<crate::codebase::dependencies::extract::CallableId>,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspace_resolution::IndexedWorkspaceMap,
    graph_files: &GraphFiles,
    interner: &PathInterner,
) -> (PathBuf, Vec<Edge>, Vec<(String, ImportTarget)>) {
    let mut edges = Vec::new();
    let mut targets = Vec::new();
    for imp in &facts.imports {
        if imp.computed {
            continue;
        }
        let kind = graph_edge_kind_for_extracted_import(imp, facts, reachable);
        let edge_kind = kind.unwrap_or_else(|| edge_kind_for_import(imp));
        let classification = resolver.classify_import(&imp.specifier, path, workspace, graph_files);
        let resolved = resolved_import_path(edge_kind, &classification, imp, graph_files);
        let recorded = import_target_for(
            resolved,
            &classification,
            resolver,
            path,
            &imp.specifier,
            workspace,
            graph_files,
        );
        targets.push((imp.specifier.clone(), recorded));
        if let Some(kind) = kind {
            if let Some(edge) = import_edge(interner, path, kind, imp, resolved, &classification) {
                edges.push(edge);
            }
        }
    }
    for call in vitest_import_calls(facts) {
        if call.computed || call.receiver != VitestReceiver::Direct {
            continue;
        }
        let classification = resolver.classify_import(call.specifier, path, workspace, graph_files);
        let resolved = classification
            .resolver_path()
            .and_then(|target| graph_files.visible_path(target))
            .or_else(|| {
                classification
                    .workspace_path()
                    .and_then(|target| graph_files.visible_path(target))
            });
        let recorded = import_target_for(
            resolved,
            &classification,
            resolver,
            path,
            call.specifier,
            workspace,
            graph_files,
        );
        targets.push((call.specifier.to_owned(), recorded));
    }
    (path.to_path_buf(), edges, targets)
}

fn import_target_for(
    resolved: Option<&Path>,
    classification: &crate::codebase::ts_resolver::ImportClassification,
    resolver: &dyn ImportResolution,
    importer: &Path,
    specifier: &str,
    workspace: &crate::codebase::workspace_resolution::IndexedWorkspaceMap,
    graph_files: &GraphFiles,
) -> ImportTarget {
    if let Some(target) = resolved {
        return ImportTarget::File(target.to_path_buf());
    }
    // Static workspace imports keep their visible package file here. The import
    // edge for that path is recorded by the workspace pass.
    if let Some(target) = classification
        .workspace_path()
        .and_then(|target| graph_files.visible_path(target))
    {
        return ImportTarget::File(target.to_path_buf());
    }
    if let Some(asset) = existing_relative_asset(importer, specifier) {
        return ImportTarget::File(asset);
    }
    if specifier_is_project(resolver, importer, specifier, classification, workspace) {
        ImportTarget::Unresolved
    } else {
        ImportTarget::External
    }
}

fn existing_relative_asset(importer: &Path, specifier: &str) -> Option<PathBuf> {
    let path_part = specifier.split(['?', '#']).next().unwrap_or(specifier);
    if !(path_part.starts_with("./") || path_part.starts_with("../")) {
        return None;
    }
    let candidate =
        crate::codebase::ts_resolver::normalize_path(&importer.parent()?.join(path_part));
    (!is_indexable(&candidate) && candidate.is_file()).then_some(candidate)
}

fn specifier_is_project(
    resolver: &dyn ImportResolution,
    importer: &Path,
    specifier: &str,
    classification: &crate::codebase::ts_resolver::ImportClassification,
    workspace: &crate::codebase::workspace_resolution::IndexedWorkspaceMap,
) -> bool {
    specifier_is_local(specifier)
        || resolver.matches_paths_alias(specifier, importer)
        || classification.workspace_path().is_some()
        || workspace.recognizes_specifier_from(specifier, importer)
}

fn specifier_is_local(specifier: &str) -> bool {
    specifier.starts_with(['.', '/', '#', '\\']) || specifier_has_drive(specifier)
}

fn specifier_has_drive(specifier: &str) -> bool {
    let bytes = specifier.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
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
