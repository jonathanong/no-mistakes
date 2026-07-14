fn collect_project_server_route_defs(
    root: &Path,
    all_files: &[PathBuf],
    tsconfig: &TsConfig,
    route_globset: &GlobSet,
    facts: Option<&dyn TsFactLookup>,
    test_filter: Option<&crate::codebase::test_filter::TestFileFilter>,
) -> Vec<(PathBuf, String)> {
    let route_files: Vec<PathBuf> = all_files
        .par_iter()
        .filter(|path| {
            path.strip_prefix(root)
                .map(|rel| route_globset.is_match(rel))
                .unwrap_or(false)
                && !test_filter.is_some_and(|filter| filter.is_match(root, path.as_path()))
        })
        .cloned()
        .collect();

    let server_route_plan = TsFactPlan {
        server_routes: true,
        ..TsFactPlan::default()
    };
    if let Some(facts) = facts.filter(|facts| facts.covers_ts_fact_plan(server_route_plan)) {
        return crate::server_routes::route_defs_from_prepared_facts(
            root,
            tsconfig,
            route_files.iter().filter_map(|path| {
                facts
                    .get_ts_facts(path)
                    .and_then(|facts| facts.server_routes.as_ref())
                    .cloned()
                    .map(|facts| (path.clone(), facts))
            }),
        );
    }

    crate::server_routes::route_defs_from_files(root, &route_files, tsconfig)
}

fn compile_project_route_globset(project_route_globs: &[String]) -> Option<GlobSet> {
    if project_route_globs.is_empty() {
        return None;
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in project_route_globs {
        let Ok(glob) = GlobBuilder::new(pattern).literal_separator(false).build() else {
            return None;
        };
        builder.add(glob);
    }
    Some(
        builder
            .build()
            .expect("globset with validated project route globs should build"),
    )
}

fn collect_backend_routes_from_graph_inputs(
    root: &Path,
    all_files: &[PathBuf],
    register_object: &str,
    pattern_globset: &GlobSet,
    facts: Option<&dyn TsFactLookup>,
    test_filter: Option<&crate::codebase::test_filter::TestFileFilter>,
) -> Vec<(PathBuf, String)> {
    let route_files: Vec<PathBuf> = all_files
        .par_iter()
        .filter(|path| {
            path.strip_prefix(root)
                .map(|rel| pattern_globset.is_match(rel))
                .unwrap_or(false)
                && !test_filter.is_some_and(|filter| filter.is_match(root, path.as_path()))
        })
        .cloned()
        .collect();
    let backend_plan = TsFactPlan {
        backend_routes: true,
        ..TsFactPlan::default()
    };
    if let Some(facts) = facts.filter(|facts| facts.covers_ts_fact_plan(backend_plan)) {
        return route_files
            .par_iter()
            .filter_map(|path| {
                facts
                    .get_ts_facts(path)
                    .map(|file_facts| (path, file_facts))
            })
            .flat_map(|(path, file_facts)| {
                file_facts
                    .backend_routes
                    .iter()
                    .filter(|route| route.register_object == register_object)
                    .map(|route| ((*path).clone(), route.route.clone()))
                    .collect::<Vec<_>>()
            })
            .collect();
    }

    crate::codebase::ts_routes::defs_backend::collect_backend_routes_from_files(
        root,
        &route_files,
        register_object,
        pattern_globset,
    )
}
