fn collect_workspace_manifest_edges(
    all_files: &[PathBuf],
    workspace: &crate::codebase::workspaces::WorkspaceMap,
    graph_files: &GraphFiles,
) -> Vec<Edge> {
    let workspace_entries: HashMap<_, _> = workspace
        .packages
        .iter()
        .filter_map(|package| {
            let entry = package.entry.as_ref()?;
            graph_files
                .is_visible(entry)
                .then(|| (package.name.as_str(), entry.clone()))
        })
        .collect();
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
                let target = workspace_entries
                    .get(name.as_str())
                    .map(|entry| NodeId::File(entry.clone()))
                    .unwrap_or_else(|| NodeId::Module(name.clone()));
                edges.push((
                    NodeId::File(path.clone()),
                    target,
                    EdgeKind::PackageDependency,
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
