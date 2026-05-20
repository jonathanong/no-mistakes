fn collect_parsed_imports_from_facts<'a>(
    files: &'a [PathBuf],
    facts: &'a TsFactMap,
) -> ParsedImports<'a> {
    files
        .par_iter()
        .filter_map(|path| {
            facts
                .get(path)
                .map(|file_facts| (path, file_facts.imports.as_slice()))
        })
        .collect()
}

fn collect_import_edges(
    parsed_imports: &ParsedImports<'_>,
    resolver: &ImportResolver<'_>,
    graph_files: &GraphFiles,
) -> Vec<Edge> {
    parsed_imports
        .par_iter()
        .flat_map_iter(|(path, imports)| {
            imports
                .iter()
                .filter_map(|imp| {
                    resolver.resolve(&imp.specifier, path).and_then(|target| {
                        if !graph_files.is_visible(&target) {
                            return None;
                        }
                        let kind = edge_kind_for_import(imp);
                        Some((NodeId::File((*path).clone()), NodeId::File(target), kind))
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn collect_workspace_edges(
    parsed_imports: &ParsedImports<'_>,
    _resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
    graph_files: &GraphFiles,
) -> Vec<Edge> {
    if workspace.packages.is_empty() {
        return vec![];
    }

    parsed_imports
        .par_iter()
        .flat_map_iter(|(path, imports)| {
            imports
                .iter()
                .filter_map(|imp| {
                    let spec = &imp.specifier;
                    if spec.starts_with('.') {
                        return None;
                    }
                    workspace.resolve_specifier(spec).and_then(|entry| {
                        if !graph_files.is_visible(&entry) {
                            return None;
                        }
                        Some((
                            NodeId::File((*path).clone()),
                            NodeId::File(entry),
                            EdgeKind::WorkspaceImport,
                        ))
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn edge_kind_for_import(import: &ExtractedImport) -> EdgeKind {
    match import.kind {
        ImportKind::Static => EdgeKind::Import,
        ImportKind::Type => EdgeKind::TypeImport,
        ImportKind::Dynamic => EdgeKind::DynamicImport,
        ImportKind::Require => EdgeKind::Require,
    }
}

fn collect_workspace_manifest_edges(
    all_files: &[PathBuf],
    workspace: &crate::codebase::workspaces::WorkspaceMap,
    graph_files: &GraphFiles,
) -> Vec<Edge> {
    if workspace.packages.is_empty() {
        return vec![];
    }

    all_files
        .par_iter()
        .flat_map_iter(|path| {
            let mut edges = Vec::new();
            if path.file_name().and_then(|name| name.to_str()) != Some("package.json") {
                return edges;
            }
            let Ok(source) = std::fs::read_to_string(path) else {
                return edges;
            };
            let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&source) else {
                return edges;
            };
            for name in package_dependency_names(&package_json) {
                let entry = workspace
                    .packages
                    .iter()
                    .find(|package| package.name == name)
                    .and_then(|package| package.entry.as_ref());
                let Some(entry) = entry else {
                    continue;
                };
                if !graph_files.is_visible(entry) {
                    continue;
                }
                edges.push((
                    NodeId::File(path.clone()),
                    NodeId::File(entry.clone()),
                    EdgeKind::WorkspaceImport,
                ));
            }
            edges
        })
        .collect()
}

fn package_dependency_names(package_json: &serde_json::Value) -> Vec<String> {
    let mut names = Vec::new();
    for field in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ] {
        let Some(deps) = package_json.get(field).and_then(|value| value.as_object()) else {
            continue;
        };
        for (name, version) in deps {
            if version.as_str().is_some() {
                names.push(name.clone());
            }
        }
    }
    names.sort();
    names.dedup();
    names
}
