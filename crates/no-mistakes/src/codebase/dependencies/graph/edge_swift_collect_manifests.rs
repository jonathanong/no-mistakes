/// Package manifests and resolved pins affect every target in their own package.
/// Keep these as ordinary canonical graph edges: test planning can then start at a
/// changed dependency file and use the same reverse traversal as source changes.
fn collect_swift_manifest_edges(
    facts: &crate::codebase::swift::SwiftFactMap,
    all_files: &[PathBuf],
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
) {
    for package in &facts.packages {
        let manifest = package.package_root.join("Package.swift");
        if !all_files.contains(&manifest) {
            continue;
        }
        for file in facts.files.values().filter(|file| {
            swift_owning_package(facts, &file.path)
                .is_some_and(|owner| owner.package_root == package.package_root)
        }) {
            if file.path != manifest {
                edges.push((
                    NodeId::file_in(interner, &file.path),
                    NodeId::file_in(interner, &manifest),
                    EdgeKind::SwiftPackageDependency,
                ));
            }
        }
        let resolved = package.package_root.join("Package.resolved");
        if all_files.contains(&resolved) {
            edges.push((
                NodeId::file_in(interner, &manifest),
                NodeId::file_in(interner, &resolved),
                EdgeKind::SwiftPackageDependency,
            ));
        }
        for local in &package.local_package_paths {
            let dependency_manifest = crate::codebase::ts_resolver::normalize_path(
                &package.package_root.join(local).join("Package.swift"),
            );
            if all_files.contains(&dependency_manifest) {
                for file in facts.files.values().filter(|file| {
                    swift_owning_package(facts, &file.path)
                        .is_some_and(|owner| owner.package_root == package.package_root)
                }) {
                    edges.push((
                        NodeId::file_in(interner, &file.path),
                        NodeId::file_in(interner, &dependency_manifest),
                        EdgeKind::SwiftPackageDependency,
                    ));
                }
            }
        }
    }
}

fn collect_swift_http_edges(
    route_def_inputs: SwiftRouteDefInputs<'_>,
    facts: &crate::codebase::swift::SwiftFactMap,
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
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
                        NodeId::file_in(interner, file.path.as_path()),
                        NodeId::file_in(interner, def_file),
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
    let session = inputs.session;
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
        route_defs.extend(collect_project_server_route_defs(ProjectRouteDefInputs {
            root,
            all_files,
            tsconfig,
            tsconfig_catalog,
            route_globset,
            facts,
            test_filter: config_options.test_filter.as_ref(),
            session,
        }));
    }
    route_defs.extend(collect_next_route_handler_defs(root, all_files, config_options));
    route_defs.sort();
    route_defs.dedup();
    route_defs
}

fn push_swift_file_edges(
    edges: &mut Vec<Edge>,
    source: &Path,
    target_files: &std::collections::BTreeSet<PathBuf>,
    kind: EdgeKind,
    interner: &PathInterner,
) {
    for target in target_files {
        if target != source {
            edges.push((
                NodeId::file_in(interner, source),
                NodeId::file_in(interner, target),
                kind,
            ));
        }
    }
}
