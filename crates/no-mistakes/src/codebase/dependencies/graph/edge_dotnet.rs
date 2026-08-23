fn collect_dotnet_edges(
    root: &Path,
    all_files: &[PathBuf],
    config_options: Option<&GraphConfigOptions>,
    prepared_facts: Option<&crate::codebase::dotnet::DotnetFactMap>,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
    interner: &PathInterner,
) -> Vec<Edge> {
    let Some(config_options) = config_options else {
        return Vec::new();
    };
    if config_options.dotnet_projects.is_empty() {
        return Vec::new();
    }
    let owned_facts = prepared_facts.is_none().then(|| {
        crate::codebase::dotnet::collect_dotnet_facts_with_sources(
            root,
            all_files,
            &config_options.dotnet_projects,
            sources,
        )
    });
    let facts = prepared_facts
        .or(owned_facts.as_ref())
        .expect("Dotnet facts are prepared or collected");
    if facts.files.is_empty() {
        return Vec::new();
    }

    let mut edges = Vec::new();
    collect_dotnet_using_edges(facts, &mut edges, interner);
    collect_dotnet_reference_edges(facts, &mut edges, interner);
    collect_dotnet_project_edges(facts, &mut edges, interner);
    collect_dotnet_route_edges(root, facts, config_options, &mut edges, interner);
    edges
}

fn collect_dotnet_route_edges(
    root: &Path,
    facts: &crate::codebase::dotnet::DotnetFactMap,
    options: &GraphConfigOptions,
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
) {
    for file in facts.files.values() {
        if !dotnet_route_file_allowed(root, &file.path, options) {
            continue;
        }
        for (_, handler) in &file.route_handlers {
            let name = handler.rsplit('.').next().unwrap_or(handler);
            if let Some(targets) = facts.methods.get(name) {
                push_dotnet_file_edges(edges, &file.path, targets, EdgeKind::RouteRef, interner);
            }
        }
    }
}

fn dotnet_route_file_allowed(root: &Path, path: &Path, options: &GraphConfigOptions) -> bool {
    let Some(globset) = options.project_route_globset.as_ref() else {
        return true;
    };
    let rel = path.strip_prefix(root).unwrap_or(path);
    globset.is_match(rel.to_string_lossy().as_ref())
}

fn collect_dotnet_using_edges(
    facts: &crate::codebase::dotnet::DotnetFactMap,
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
) {
    for file in facts.files.values() {
        for using in &file.usings {
            if let Some(target_files) = facts.files_by_namespace.get(using) {
                push_dotnet_file_edges(
                    edges,
                    &file.path,
                    target_files,
                    EdgeKind::DotnetUsing,
                    interner,
                );
            }
        }
    }
}

fn collect_dotnet_reference_edges(
    facts: &crate::codebase::dotnet::DotnetFactMap,
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
) {
    for file in facts.files.values() {
        for reference in &file.references {
            if let Some(target_files) = facts.declarations.get(reference) {
                push_dotnet_file_edges(
                    edges,
                    &file.path,
                    target_files,
                    EdgeKind::DotnetReference,
                    interner,
                );
            }
        }
    }
}

fn collect_dotnet_project_edges(
    facts: &crate::codebase::dotnet::DotnetFactMap,
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
) {
    for project in facts.projects.values() {
        let Some(source_files) = facts.files_by_project.get(&project.project_path) else {
            continue;
        };
        let test_files = source_files.iter().filter(|path| {
            facts
                .files
                .get(*path)
                .is_some_and(|file| file.has_xunit_tests)
        });
        for reference in &project.project_references {
            if let Some(target_files) = facts.files_by_project.get(reference) {
                for source in test_files.clone() {
                    push_dotnet_file_edges(
                        edges,
                        source,
                        target_files,
                        EdgeKind::DotnetProjectDependency,
                        interner,
                    );
                }
            }
        }
    }
}

fn push_dotnet_file_edges(
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
