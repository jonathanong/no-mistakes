fn collect_workspace_manifest_edges(
    all_files: &[PathBuf],
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
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
            let Some(dependency_names) = workspace.manifest_dependency_names(path) else {
                return edges;
            };
            for name in dependency_names {
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



#[cfg(test)]
mod edge_package_manifest_tests;
