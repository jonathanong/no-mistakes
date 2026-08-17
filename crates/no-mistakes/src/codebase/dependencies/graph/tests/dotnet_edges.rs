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
        queue_enqueues: Vec::new(),
        queue_workers: Vec::new(),
        queue_cluster: None,
        queue_glob_clusters: HashMap::new(),
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
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());
}

#[test]
fn dotnet_project_edges_skip_missing_sources_and_references() {
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
    assert!(edges.is_empty());

    facts
        .files_by_project
        .insert(test_project, [test_file.clone()].into_iter().collect());
    facts
        .files_by_project
        .insert(app_project, [app_file.clone()].into_iter().collect());
    collect_dotnet_project_edges(
        &facts,
        &mut edges,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    assert_eq!(
        edges,
        vec![(
            NodeId::file(test_file),
            NodeId::file(app_file),
            EdgeKind::DotnetProjectDependency
        )]
    );
}
