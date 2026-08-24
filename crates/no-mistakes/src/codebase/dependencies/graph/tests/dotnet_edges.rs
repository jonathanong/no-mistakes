use super::*;

fn empty_options() -> GraphConfigOptions {
    GraphConfigOptions {
        route: Default::default(),
        queue: Default::default(),
        http_route: Default::default(),
        http_call: Default::default(),
        project_route_globset: None,
        test_filter: None,
        rewrites: Vec::new(),
        queue_project_factory_names: Vec::new(),
        dotnet_projects: Vec::new(),
        swift_packages: Vec::new(),
        python_packages: Vec::new(),
        go_modules: Vec::new(),
        rust_packages: Vec::new(),
        rails_apps: Vec::new(),
        php_apps: Vec::new(),
        php_framework: None,
        java_packages: Vec::new(),
        kotlin_packages: Vec::new(),
        elixir_apps: Vec::new(),
        dart_packages: Vec::new(),
        queue_enqueues: Vec::new(),
        queue_workers: Vec::new(),
        queue_cluster: None,
        queue_glob_clusters: HashMap::new(),
        trpc_routers: Vec::new(),
        terraform: Default::default(),
        ci: crate::config::v2::schema::CiConfig::default(),
    }
}

#[test]
fn dotnet_edges_return_empty_without_config_or_files() {
    let root = p("/repo");
    assert!(collect_dotnet_edges(
        &root,
        &[],
        None,
        None,
        None,
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());

    let mut options = empty_options();
    options
        .dotnet_projects
        .push(crate::codebase::dotnet::DotnetConfigProject {
            name: "missing".to_string(),
            project: "Missing.csproj".to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            test: true,
        });

    assert!(collect_dotnet_edges(
        &root,
        &[],
        Some(&options),
        None,
        None,
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());
}

#[test]
fn dotnet_project_edges_emit_project_references_without_parseable_sources() {
    let test_project = p("/repo/tests/App.Tests/App.Tests.csproj");
    let app_project = p("/repo/src/App/App.csproj");
    let test_file = p("/repo/tests/App.Tests/FeedServiceTests.cs");
    let app_file = p("/repo/src/App/FeedService.cs");

    let mut facts = crate::codebase::dotnet::DotnetFactMap::default();
    facts.projects.insert(
        test_project.clone(),
        crate::codebase::dotnet::DotnetProjectFacts {
            project_path: test_project.clone(),
            project_references: [app_project.clone(), p("/repo/src/Missing/Missing.csproj")]
                .into_iter()
                .collect(),
            ..Default::default()
        },
    );
    facts.files.insert(
        test_file.clone(),
        crate::codebase::dotnet::DotnetFileFacts {
            path: test_file.clone(),
            has_xunit_tests: true,
            ..Default::default()
        },
    );

    let mut edges = Vec::new();
    collect_dotnet_project_edges(
        &facts,
        &mut edges,
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert_eq!(
        edges,
        vec![
            (
                NodeId::file(test_project.clone()),
                NodeId::file(app_project.clone()),
                EdgeKind::DotnetProjectDependency,
            ),
            (
                NodeId::file(test_project.clone()),
                NodeId::file(p("/repo/src/Missing/Missing.csproj")),
                EdgeKind::DotnetProjectDependency,
            ),
        ]
    );

    edges.clear();
    facts.files_by_project.insert(
        test_project.clone(),
        [test_file.clone()].into_iter().collect(),
    );
    facts.files_by_project.insert(
        app_project.clone(),
        [app_file.clone()].into_iter().collect(),
    );
    collect_dotnet_project_edges(
        &facts,
        &mut edges,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    assert_eq!(
        edges,
        vec![
            (
                NodeId::file(test_project),
                NodeId::file(app_project),
                EdgeKind::DotnetProjectDependency,
            ),
            (
                NodeId::file(p("/repo/tests/App.Tests/App.Tests.csproj")),
                NodeId::file(p("/repo/src/Missing/Missing.csproj")),
                EdgeKind::DotnetProjectDependency,
            ),
            (
                NodeId::file(test_file),
                NodeId::file(app_file),
                EdgeKind::DotnetProjectDependency,
            ),
        ]
    );
}

#[test]
fn dotnet_dependency_files_connect_only_actual_project_consumers() {
    let root = p("/repo");
    let project = root.join("src/App/App.csproj");
    let source = root.join("src/App/App.cs");
    let central = root.join("Directory.Packages.props");
    let lock = root.join("src/App/packages.lock.json");
    let mut facts = crate::codebase::dotnet::DotnetFactMap::default();
    facts.projects.insert(
        project.clone(),
        crate::codebase::dotnet::DotnetProjectFacts {
            project_path: project.clone(),
            project_dir: root.join("src/App"),
            compile_files: [source.clone()].into_iter().collect(),
            package_references: ["Example.Package".to_string()].into_iter().collect(),
            ..Default::default()
        },
    );

    let mut edges = Vec::new();
    collect_dotnet_dependency_file_edges(
        &facts,
        &[central.clone(), lock.clone()],
        None,
        &mut edges,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    assert!(edges.contains(&(
        NodeId::file(source),
        NodeId::file(project.clone()),
        EdgeKind::DotnetProjectDependency
    )));
    assert!(edges.contains(&(
        NodeId::file(project.clone()),
        NodeId::file(central),
        EdgeKind::DotnetProjectDependency
    )));
    assert!(edges.contains(&(
        NodeId::file(project),
        NodeId::file(lock),
        EdgeKind::DotnetProjectDependency
    )));
}

#[test]
fn dotnet_central_package_edges_use_only_the_nearest_ancestor() {
    let root = p("/repo");
    let project = root.join("apps/App/App.csproj");
    let root_central = root.join("Directory.Packages.props");
    let app_central = root.join("apps/Directory.Packages.props");
    let mut facts = crate::codebase::dotnet::DotnetFactMap::default();
    facts.projects.insert(
        project.clone(),
        crate::codebase::dotnet::DotnetProjectFacts {
            project_path: project.clone(),
            project_dir: root.join("apps/App"),
            package_references: ["Example.Package".to_string()].into_iter().collect(),
            ..Default::default()
        },
    );

    let mut edges = Vec::new();
    collect_dotnet_dependency_file_edges(
        &facts,
        &[root_central.clone(), app_central.clone()],
        None,
        &mut edges,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    assert!(edges.contains(&(
        NodeId::file(project.clone()),
        NodeId::file(app_central),
        EdgeKind::DotnetProjectDependency,
    )));
    assert!(!edges.contains(&(
        NodeId::file(project),
        NodeId::file(root_central),
        EdgeKind::DotnetProjectDependency,
    )));
}

#[test]
fn dotnet_central_package_edges_include_projects_without_package_references() {
    let root = p("/repo");
    let project = root.join("apps/App/App.csproj");
    let central = root.join("Directory.Packages.props");
    let mut facts = crate::codebase::dotnet::DotnetFactMap::default();
    facts.projects.insert(
        project.clone(),
        crate::codebase::dotnet::DotnetProjectFacts {
            project_path: project.clone(),
            project_dir: root.join("apps/App"),
            ..Default::default()
        },
    );

    let mut edges = Vec::new();
    collect_dotnet_dependency_file_edges(
        &facts,
        std::slice::from_ref(&central),
        None,
        &mut edges,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    assert!(edges.contains(&(
        NodeId::file(project),
        NodeId::file(central),
        EdgeKind::DotnetProjectDependency,
    )));
}

#[test]
fn dotnet_central_package_import_edges_follow_configured_project_closure() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase-analysis/dotnet-central-imports/fixture"),
    );
    let parent = root.join("Directory.Packages.props");
    let nested = root.join("nested/Directory.Packages.props");
    let deeper = root.join("nested/deeper/Directory.Packages.props");
    let malformed = root.join("malformed/Directory.Packages.props");
    let unrelated = root.join("unrelated/Directory.Packages.props");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::ts_source::SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&files),
    ));
    let mut facts = crate::codebase::dotnet::DotnetFactMap::default();
    for project in [
        root.join("nested/deeper/Configured.csproj"),
        root.join("malformed/Configured.csproj"),
    ] {
        facts.projects.insert(
            project.clone(),
            crate::codebase::dotnet::DotnetProjectFacts {
                project_dir: project.parent().unwrap().to_path_buf(),
                project_path: project,
                ..Default::default()
            },
        );
    }

    let mut edges = Vec::new();
    collect_dotnet_central_import_edges(
        &facts,
        &files,
        Some(&sources),
        &mut edges,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    assert_eq!(
        edges,
        vec![
            (NodeId::file(malformed), NodeId::file(parent.clone()), EdgeKind::DotnetProjectDependency),
            (NodeId::file(deeper), NodeId::file(nested.clone()), EdgeKind::DotnetProjectDependency),
            (NodeId::file(nested), NodeId::file(parent), EdgeKind::DotnetProjectDependency),
        ]
    );
    let unrelated = NodeId::file(unrelated);
    assert!(edges.iter().all(|(from, _, _)| from != &unrelated));
}

#[test]
fn aspnet_map_get_emits_route_ref_to_handler_file() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/dotnet-aspnet-routes/fixture"),
    );
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let mut options = empty_options();
    options.dotnet_projects =
        crate::codebase::dotnet::configured_projects(&root, &config.tests.dotnet);
    let edges = collect_dotnet_edges(
        &root,
        &all_files,
        Some(&options),
        None,
        None,
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RouteRef
            && from
                .as_file()
                .is_some_and(|path| path.ends_with("Program.cs"))
            && to
                .as_file()
                .is_some_and(|path| path.ends_with("UserHandlers.cs"))
    }));
    assert!(edges.iter().all(|(from, _, kind)| {
        *kind != EdgeKind::RouteRef
            || from
                .as_file()
                .is_none_or(|path| !path.ends_with("Computed.cs"))
    }));
}

#[test]
fn aspnet_route_globs_exclude_registration_files() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/dotnet-aspnet-routes/fixture"),
    );
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let mut options = empty_options();
    options.dotnet_projects =
        crate::codebase::dotnet::configured_projects(&root, &config.tests.dotnet);
    let mut builder = globset::GlobSetBuilder::new();
    builder.add(globset::Glob::new("**/UsersController.cs").unwrap());
    options.project_route_globset = Some(builder.build().unwrap());
    let edges = collect_dotnet_edges(
        &root,
        &all_files,
        Some(&options),
        None,
        None,
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(edges.iter().all(|(from, _, kind)| {
        *kind != EdgeKind::RouteRef
            || from
                .as_file()
                .is_none_or(|path| !path.ends_with("Program.cs"))
    }));

    let mut builder = globset::GlobSetBuilder::new();
    builder.add(globset::Glob::new("**/Program.cs").unwrap());
    options.project_route_globset = Some(builder.build().unwrap());
    let edges = collect_dotnet_edges(
        &root,
        &all_files,
        Some(&options),
        None,
        None,
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::RouteRef
            && from
                .as_file()
                .is_some_and(|path| path.ends_with("Program.cs"))
            && to
                .as_file()
                .is_some_and(|path| path.ends_with("UserHandlers.cs"))
    }));
}

#[test]
fn dotnet_symbols_stay_inside_source_project_and_project_references() {
    let app_project = p("/repo/app/App.csproj");
    let unrelated_project = p("/repo/other/Other.csproj");
    let source = p("/repo/app/Caller.cs");
    let declared = p("/repo/app/Service.cs");
    let unrelated = p("/repo/other/Service.cs");
    let mut facts = crate::codebase::dotnet::DotnetFactMap::default();
    facts.files.insert(source.clone(), crate::codebase::dotnet::DotnetFileFacts { path: source.clone(), project: Some(app_project.clone()), usings: vec!["App.Services".to_string()], references: vec!["Service".to_string()], ..Default::default() });
    for (path, project) in [(&declared, app_project.clone()), (&unrelated, unrelated_project)] {
        facts.files.insert(path.clone(), crate::codebase::dotnet::DotnetFileFacts { path: path.clone(), project: Some(project), namespace: Some("App.Services".to_string()), declarations: vec!["Service".to_string()], ..Default::default() });
    }
    facts.files_by_namespace.insert("App.Services".to_string(), [declared.clone(), unrelated.clone()].into_iter().collect());
    facts.declarations.insert("Service".to_string(), [declared.clone(), unrelated.clone()].into_iter().collect());
    let mut edges = Vec::new();
    let interner = crate::codebase::analysis_session::PathInterner::new();
    collect_dotnet_using_edges(&facts, &mut edges, &interner);
    collect_dotnet_reference_edges(&facts, &mut edges, &interner);
    assert!(edges.iter().any(|(_, target, _)| target.as_file() == Some(&declared)));
    assert!(!edges.iter().any(|(_, target, _)| target.as_file() == Some(&unrelated)));
}
