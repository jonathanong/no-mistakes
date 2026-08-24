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
    collect_dotnet_dependency_file_edges(facts, all_files, &mut edges, interner);
    collect_dotnet_route_edges(root, facts, config_options, &mut edges, interner);
    edges
}

/// Model the project file, central package versions, and per-project lockfile as
/// canonical dependencies. A change to any of those files is therefore traced
/// through only the projects which consume it instead of forcing a framework
/// fallback.
fn collect_dotnet_dependency_file_edges(
    facts: &crate::codebase::dotnet::DotnetFactMap,
    all_files: &[PathBuf],
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
) {
    for project in facts.projects.values() {
        for source in &project.compile_files {
            edges.push((
                NodeId::file_in(interner, source),
                NodeId::file_in(interner, &project.project_path),
                EdgeKind::DotnetProjectDependency,
            ));
        }

        let lock = project.project_dir.join("packages.lock.json");
        if all_files.contains(&lock) {
            edges.push((
                NodeId::file_in(interner, &project.project_path),
                NodeId::file_in(interner, &lock),
                EdgeKind::DotnetProjectDependency,
            ));
        }

        if project.package_references.is_empty() {
            continue;
        }
        let central = all_files
            .iter()
            .filter(|path| {
                path.file_name().and_then(|name| name.to_str()) == Some("Directory.Packages.props")
                    && project
                        .project_path
                        .starts_with(path.parent().unwrap_or(path))
            })
            .max_by_key(|path| {
                path.parent()
                    .map_or(0, |parent| parent.components().count())
            });
        if let Some(central) = central {
            edges.push((
                NodeId::file_in(interner, &project.project_path),
                NodeId::file_in(interner, central),
                EdgeKind::DotnetProjectDependency,
            ));
        }
    }
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
        let allowed_projects = file
            .project
            .as_ref()
            .map(|project| dotnet_project_scope(facts, project))
            .unwrap_or_default();
        for using in &file.usings {
            if let Some(target_files) = facts.files_by_namespace.get(using) {
                let target_files = scoped_dotnet_target_files(facts, target_files, &allowed_projects);
                push_dotnet_file_edges(
                    edges,
                    &file.path,
                    &target_files,
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
        let allowed_projects = file
            .project
            .as_ref()
            .map(|project| dotnet_project_scope(facts, project))
            .unwrap_or_default();
        for reference in &file.references {
            if let Some(target_files) = facts.declarations.get(reference) {
                let target_files = scoped_dotnet_target_files(facts, target_files, &allowed_projects);
                push_dotnet_file_edges(
                    edges,
                    &file.path,
                    &target_files,
                    EdgeKind::DotnetReference,
                    interner,
                );
            }
        }
    }
}
