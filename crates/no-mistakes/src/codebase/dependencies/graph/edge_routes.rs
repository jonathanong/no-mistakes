fn collect_route_edges(
    root: &Path,
    tsconfig: &TsConfig,
    all_files: &[PathBuf],
    facts: Option<&dyn TsFactLookup>,
    config_options: Option<&GraphConfigOptions>,
) -> Vec<Edge> {
    use crate::codebase::ts_routes::defs_frontend;
    use crate::codebase::ts_resolver::ImportResolver;
    use globset::{GlobBuilder, GlobSetBuilder};

    let Some(config_options) = config_options else {
        return vec![];
    };
    let opts = &config_options.route;
    let project_route_globset = config_options.project_route_globset.as_ref();
    let has_project_routes = project_route_globset.is_some();

    if !has_project_routes
        && (opts.backend_pattern.is_empty() || opts.backend_register_object.is_empty())
        && opts.frontend_root.is_empty()
    {
        return vec![];
    }

    let mut all_defs: Vec<(PathBuf, String)> = Vec::new();
    let backend_defs =
        if !opts.backend_pattern.is_empty() && !opts.backend_register_object.is_empty() {
            match GlobBuilder::new(&opts.backend_pattern)
                .literal_separator(false)
                .build()
            {
                Ok(glob) => {
                    let mut gb = GlobSetBuilder::new();
                    gb.add(glob);
                    let gs = gb
                        .build()
                        .expect("globset with one validated backend route glob should build");
                    collect_backend_routes_from_graph_inputs(
                        root,
                        all_files,
                        &opts.backend_register_object,
                        &gs,
                        facts,
                        config_options.test_filter.as_ref(),
                    )
                }
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        };
    all_defs.extend(backend_defs);
    if has_project_routes {
        all_defs.extend(collect_project_server_route_defs(
            root,
            all_files,
            tsconfig,
            project_route_globset.expect("project route globset checked above"),
            config_options.test_filter.as_ref(),
        ));
    }
    if !opts.frontend_root.is_empty() {
        let frontend_abs = root.join(&opts.frontend_root);
        all_defs.extend(defs_frontend::collect_frontend_routes_from_files(
            &frontend_abs,
            all_files,
        ));
    }
    all_defs.sort();
    all_defs.dedup();
    let virtual_defs = crate::routes::rewrites::expand_rewrites_as_tuples(
        &config_options.rewrites,
        &all_defs,
    );
    all_defs.extend(virtual_defs);
    all_defs.sort();
    all_defs.dedup();
    if all_defs.is_empty() {
        return vec![];
    }

    let mut pattern_to_files: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for (file, pattern) in &all_defs {
        pattern_to_files
            .entry(pattern.clone())
            .or_default()
            .push(file.clone());
    }
    let mut all_patterns: Vec<String> = pattern_to_files.keys().cloned().collect();
    all_patterns.sort();
    let resolver = ImportResolver::new(tsconfig);

    let backend_prefixes = route_backend_prefixes(config_options);
    let backend_exact = opts.backend_exact_paths.clone();
    let has_backend_filter = !backend_prefixes.is_empty() || !backend_exact.is_empty();

    let scan_globs: Vec<String> = if opts.scan_patterns.is_empty() {
        vec![
            "**/*.tsx".to_string(),
            "**/*.ts".to_string(),
            "**/*.mts".to_string(),
        ]
    } else {
        opts.scan_patterns.clone()
    };
    let mut scan_gb = GlobSetBuilder::new();
    for glob in scan_globs.iter().filter_map(|g| Glob::new(g).ok()) {
        scan_gb.add(glob);
    }
    let scan_gs = scan_gb
        .build()
        .expect("globset with individually validated scan globs should build");

    let scan_files: Vec<PathBuf> = all_files
        .iter()
        .filter(|p| {
            p.strip_prefix(root)
                .map(|rel| scan_gs.is_match(rel))
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    let mut edges: Vec<_> = scan_files
        .into_par_iter()
        .flat_map_iter(|path| {
            let mut edges = Vec::new();
            let mut push_edges_for_refs =
                |route_patterns: Vec<&str>| {
                    for route_pattern in route_patterns {
                        if route_pattern_should_skip(
                            route_pattern,
                            &backend_prefixes,
                            &backend_exact,
                            has_backend_filter,
                            opts.frontend_root.is_empty(),
                        ) {
                            continue;
                        }
                        push_matching_route_edges(
                            &mut edges,
                            &path,
                            route_pattern,
                            &all_patterns,
                            &pattern_to_files,
                        );
                    }
                };
            if let Some(file_facts) = facts.and_then(|facts| facts.get_ts_facts(&path)) {
                push_edges_for_refs(
                    file_facts
                        .route_refs
                        .iter()
                        .map(|route_ref| route_ref.pattern.as_str())
                        .collect(),
                );
                let helper_patterns = route_helper_ref_patterns(
                    &path,
                    file_facts,
                    facts.expect("route facts are available when file facts were found"),
                    &resolver,
                );
                push_edges_for_refs(helper_patterns.iter().map(String::as_str).collect());
            }
            edges
        })
        .collect();
    edges.sort();
    edges.dedup();
    edges
}
