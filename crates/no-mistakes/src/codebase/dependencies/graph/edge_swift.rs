struct SwiftRouteDefInputs<'a> {
    root: &'a Path,
    tsconfig: &'a TsConfig,
    tsconfig_catalog: Option<&'a crate::codebase::ts_resolver::TsConfigCatalog>,
    all_files: &'a [PathBuf],
    config_options: &'a GraphConfigOptions,
    ts_facts: Option<&'a dyn TsFactLookup>,
}

fn collect_swift_edges_with_facts(
    root: &Path,
    tsconfig: &TsConfig,
    tsconfig_catalog: Option<&crate::codebase::ts_resolver::TsConfigCatalog>,
    all_files: &[PathBuf],
    config_options: Option<&GraphConfigOptions>,
    ts_facts: Option<&dyn TsFactLookup>,
    prepared_facts: Option<&crate::codebase::swift::SwiftFactMap>,
) -> Vec<Edge> {
    let Some(config_options) = config_options else {
        return Vec::new();
    };
    if config_options.swift_packages.is_empty() {
        return Vec::new();
    }
    let owned_facts = prepared_facts.is_none().then(|| {
        crate::codebase::swift::collect_swift_facts(root, all_files, &config_options.swift_packages)
    });
    let facts = prepared_facts
        .or(owned_facts.as_ref())
        .expect("Swift facts are prepared or collected");
    if facts.files.is_empty() {
        return Vec::new();
    }

    let mut edges = Vec::new();
    collect_swift_import_edges(facts, &mut edges);
    collect_swift_reference_edges(facts, &mut edges);
    collect_swift_package_edges(facts, &mut edges);
    collect_swift_http_edges(
        SwiftRouteDefInputs {
            root,
            tsconfig,
            tsconfig_catalog,
            all_files,
            config_options,
            ts_facts,
        },
        facts,
        &mut edges,
    );
    edges
}

fn collect_swift_import_edges(facts: &crate::codebase::swift::SwiftFactMap, edges: &mut Vec<Edge>) {
    for file in facts.files.values() {
        for import in &file.imports {
            if let Some(target_files) = facts.files_by_target.get(import) {
                push_swift_file_edges(edges, &file.path, target_files, EdgeKind::SwiftImport);
            }
        }
    }
}

fn collect_swift_reference_edges(
    facts: &crate::codebase::swift::SwiftFactMap,
    edges: &mut Vec<Edge>,
) {
    for file in facts.files.values() {
        for reference in &file.references {
            if let Some(target_files) = facts.declarations.get(reference) {
                push_swift_file_edges(edges, &file.path, target_files, EdgeKind::SwiftReference);
            }
        }
    }
}

fn collect_swift_package_edges(
    facts: &crate::codebase::swift::SwiftFactMap,
    edges: &mut Vec<Edge>,
) {
    for package in &facts.packages {
        for target in package.targets.values() {
            let Some(source_files) = facts.files_by_target.get(&target.name) else {
                continue;
            };
            for dependency in &target.dependencies {
                if let Some(dep_files) = facts.files_by_target.get(dependency) {
                    for source in source_files {
                        push_swift_file_edges(
                            edges,
                            source,
                            dep_files,
                            EdgeKind::SwiftPackageDependency,
                        );
                    }
                }
            }
        }
    }
}

fn collect_swift_http_edges(
    route_def_inputs: SwiftRouteDefInputs<'_>,
    facts: &crate::codebase::swift::SwiftFactMap,
    edges: &mut Vec<Edge>,
) {
    let route_defs = swift_route_defs(&route_def_inputs);
    if route_defs.is_empty() {
        return;
    }
    for file in facts.files.values() {
        for path in &file.endpoint_paths {
            for (def_file, def_pattern) in &route_defs {
                if def_file != &file.path
                    && crate::codebase::ts_routes::matcher::matches(path, def_pattern)
                {
                    edges.push((
                        NodeId::File(file.path.clone()),
                        NodeId::File(def_file.clone()),
                        EdgeKind::HttpCall,
                    ));
                }
            }
        }
    }
}

fn swift_route_defs(inputs: &SwiftRouteDefInputs<'_>) -> Vec<(PathBuf, String)> {
    let root = inputs.root;
    let tsconfig = inputs.tsconfig;
    let tsconfig_catalog = inputs.tsconfig_catalog;
    let all_files = inputs.all_files;
    let config_options = inputs.config_options;
    let facts = inputs.ts_facts;
    let mut route_defs = Vec::new();
    if let (Some(backend_pattern), Some(register_object)) = (
        resolved_backend_pattern(config_options),
        resolved_backend_register_object(config_options),
    ) {
        if let Some(gs) = compile_graph_glob(&backend_pattern) {
            route_defs.extend(collect_backend_routes_from_graph_inputs(
                root,
                all_files,
                &register_object,
                &gs,
                facts,
                config_options.test_filter.as_ref(),
            ));
        }
    }
    if let Some(route_globset) = config_options.project_route_globset.as_ref() {
        route_defs.extend(collect_project_server_route_defs(
            root,
            all_files,
            tsconfig,
            tsconfig_catalog,
            route_globset,
            facts,
            config_options.test_filter.as_ref(),
        ));
    }
    route_defs.extend(collect_next_route_handler_defs(
        root,
        all_files,
        config_options,
    ));
    route_defs.sort();
    route_defs.dedup();
    route_defs
}

fn push_swift_file_edges(
    edges: &mut Vec<Edge>,
    source: &Path,
    target_files: &std::collections::BTreeSet<PathBuf>,
    kind: EdgeKind,
) {
    for target in target_files {
        if target != source {
            edges.push((
                NodeId::File(source.to_path_buf()),
                NodeId::File(target.clone()),
                kind,
            ));
        }
    }
}
