/// Resolve every export spelling once while the graph build still owns the
/// resolver and facts. Root expansion runs after those inputs are dropped, so
/// retaining this result is what lets an uncalled exported function still
/// resolve to its canonical repository callable.
struct ExportNameEdges {
    direct: std::collections::HashSet<String>,
    stars: Vec<std::path::PathBuf>,
    namespaces: Vec<(String, std::path::PathBuf)>,
}

fn exported_names_for_file(
    path: &std::path::Path,
    edges: &FxHashMap<std::path::PathBuf, ExportNameEdges>,
    visiting: &mut std::collections::HashSet<std::path::PathBuf>,
) -> std::collections::HashSet<String> {
    if !visiting.insert(path.to_path_buf()) {
        return std::collections::HashSet::new();
    }
    let Some(file) = edges.get(path) else {
        visiting.remove(path);
        return std::collections::HashSet::new();
    };
    let mut names = file.direct.clone();
    for target in &file.stars {
        names.extend(
            exported_names_for_file(target, edges, visiting)
                .into_iter()
                .filter(|name| name != "default"),
        );
    }
    for (namespace, target) in &file.namespaces {
        names.extend(
            exported_names_for_file(target, edges, visiting)
                .into_iter()
                .map(|member| format!("{namespace}.{member}")),
        );
    }
    visiting.remove(path);
    names
}

fn populate_callable_export_resolutions(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    output: &mut FxHashMap<(std::path::PathBuf, String), ExportedCallableResolution>,
) {
    let mut export_edges = FxHashMap::default();
    for path in edge_inputs.graph_files.indexable() {
        let Some(file) = indexes.file(facts, path) else {
            continue;
        };
        let mut names = std::collections::HashSet::new();
        let mut namespaces = Vec::new();
        for (name, binding) in &file.exported {
            if binding.local == "*" {
                if let Some(target) = binding.specifier.as_deref().and_then(|specifier| {
                    resolver
                        .resolve(specifier, path)
                        .and_then(|target| edge_inputs.graph_files.visible_path(&target))
                        .map(std::path::Path::to_path_buf)
                }) {
                    namespaces.push((name.clone(), target));
                }
            } else {
                names.insert(name.clone());
            }
        }
        let stars = file
            .stars
            .iter()
            .filter_map(|specifier| {
                resolver
                    .resolve(specifier, path)
                    .and_then(|target| edge_inputs.graph_files.visible_path(&target))
                    .map(std::path::Path::to_path_buf)
            })
            .collect::<Vec<_>>();
        export_edges.insert(
            path.clone(),
            ExportNameEdges {
                direct: names,
                stars,
                namespaces,
            },
        );
    }
    let paths = export_edges.keys().cloned().collect::<Vec<_>>();
    for path in paths {
        let candidates = exported_names_for_file(
            &path,
            &export_edges,
            &mut std::collections::HashSet::new(),
        );
        for export in candidates {
            let resolution = resolve_exported_callable(
                edge_inputs,
                facts,
                resolver,
                &path,
                &export,
                indexes,
                &mut Vec::new(),
            );
            if !matches!(resolution, ExportedCallableResolution::Absent) {
                output.insert((path.clone(), export), resolution);
            }
        }
    }
}
