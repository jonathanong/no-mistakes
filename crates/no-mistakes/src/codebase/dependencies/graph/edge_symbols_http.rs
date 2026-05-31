fn collect_symbol_http_route_defs(
    root: &Path,
    all_files: &[PathBuf],
    facts: &dyn TsFactLookup,
    config_options: Option<&GraphConfigOptions>,
) -> Vec<(PathBuf, String)> {
    let Some(config_options) = config_options else {
        return Vec::new();
    };
    if resolved_backend_prefixes(config_options).is_empty() {
        return Vec::new();
    }
    let mut route_defs = match (
        resolved_backend_pattern(config_options),
        resolved_backend_register_object(config_options),
    ) {
        (Some(backend_pattern), Some(register_object)) => compile_graph_glob(&backend_pattern)
            .map(|gs| {
                collect_backend_routes_from_graph_inputs(
                    root,
                    all_files,
                    &register_object,
                    &gs,
                    Some(facts),
                    config_options.test_filter.as_ref(),
                )
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    route_defs.extend(collect_next_route_handler_defs(root, all_files, config_options));
    route_defs
}
