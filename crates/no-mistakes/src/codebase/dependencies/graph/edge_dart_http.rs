fn collect_ts_and_dart_http_call_edges(
    root: &Path,
    facts: Option<&dyn TsFactLookup>,
    files: &[(PathBuf, String)],
    graph_files: &[PathBuf],
    all_files: &[PathBuf],
    config_options: &GraphConfigOptions,
    route_defs: &[(PathBuf, String)],
    prefix_strs: &[&str],
    interner: &PathInterner,
) -> Vec<Edge> {
    use crate::codebase::ts_http_calls::extract_http_calls;
    let mut edges: Vec<Edge> = if let Some(facts) = facts {
        graph_files
            .par_iter()
            .filter_map(|caller| {
                facts
                    .get_ts_facts(caller)
                    .map(|file_facts| (caller.as_path(), file_facts.http_calls.as_slice()))
            })
            .flat_map_iter(|(caller, calls)| {
                http_edges_for_calls(caller, calls, route_defs, interner)
            })
            .collect()
    } else {
        files
            .par_iter()
            .flat_map_iter(|(caller, source)| {
                let calls = extract_http_calls(source, prefix_strs);
                http_edges_for_calls(caller, &calls, route_defs, interner)
            })
            .collect()
    };
    edges.extend(collect_dart_http_call_edges(
        root,
        all_files,
        config_options,
        route_defs,
        interner,
    ));
    edges
}

fn collect_dart_http_call_edges(
    root: &Path,
    all_files: &[PathBuf],
    config_options: &GraphConfigOptions,
    route_defs: &[(PathBuf, String)],
    interner: &PathInterner,
) -> Vec<Edge> {
    if config_options.dart_packages.is_empty() {
        return Vec::new();
    }
    let roots =
        crate::codebase::lang_frontends::configured_roots(root, &config_options.dart_packages);
    let dart_files: Vec<PathBuf> = all_files
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("dart"))
        .filter(|path| roots.iter().any(|package_root| path.starts_with(package_root)))
        .cloned()
        .collect();
    if dart_files.is_empty() {
        return Vec::new();
    }
    let store = crate::codebase::ts_source::SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&dart_files),
    ));
    dart_files
        .par_iter()
        .flat_map(|caller| {
            let Ok(source) = store.read_path(caller) else {
                return Vec::new();
            };
            let calls: Vec<_> = crate::codebase::lang_frontends::extract_http_paths(&source)
                .into_iter()
                .map(|path| crate::codebase::ts_http_calls::HttpCall { path, line: 0 })
                .collect();
            http_edges_for_calls(caller, &calls, route_defs, interner)
        })
        .collect()
}
