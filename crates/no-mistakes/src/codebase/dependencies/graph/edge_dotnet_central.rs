/// Nested central package manifests do not inherit an ancestor automatically.
/// They can, however, explicitly import one, so retain that dependency in the
/// canonical graph instead of widening every parent central change.
fn collect_dotnet_central_import_edges(
    facts: &crate::codebase::dotnet::DotnetFactMap,
    all_files: &[PathBuf],
    sources: Option<&crate::codebase::ts_source::SourceStore>,
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
) {
    let central_files: std::collections::BTreeSet<PathBuf> = all_files
        .iter()
        .filter(|path| is_central_package_file(path))
        .cloned()
        .collect();
    let mut pending = facts
        .projects
        .values()
        .filter_map(|project| nearest_central_package_file(&project.project_dir, &central_files))
        .collect::<std::collections::BTreeSet<_>>();
    let mut visited = std::collections::BTreeSet::new();
    while let Some(central) = pending.pop_first() {
        if !visited.insert(central.clone()) {
            continue;
        }
        let Some(source) = crate::codebase::ts_source::SourceStore::read_optional(sources, &central)
        else {
            continue;
        };
        let ancestors = crate::codebase::dotnet::central_ancestor_files(&central, &central_files)
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        for imported in crate::codebase::dotnet::central_package_imports(&central, &source, &central_files)
            .into_iter()
            .filter(|imported| ancestors.contains(imported))
        {
            pending.insert(imported.clone());
            edges.push((
                NodeId::file_in(interner, &central),
                NodeId::file_in(interner, &imported),
                EdgeKind::DotnetProjectDependency,
            ));
        }
    }
}

fn nearest_central_package_file(
    project_dir: &Path,
    central_files: &std::collections::BTreeSet<PathBuf>,
) -> Option<PathBuf> {
    central_files
        .iter()
        .filter(|central| project_dir.starts_with(central.parent().unwrap_or(central)))
        .max_by_key(|central| central.parent().map_or(0, |parent| parent.components().count()))
        .cloned()
}

fn is_central_package_file(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some("Directory.Packages.props")
}
