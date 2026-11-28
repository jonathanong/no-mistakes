#[test]
fn dotnet_only_edge_plan_omits_http_calls() {
    let root = lang_fixture("http-clients");
    let files = lang_files(&root);
    let options = GraphConfigOptions {
        dotnet_projects: vec![crate::codebase::dotnet::DotnetConfigProject {
            name: "api".to_string(),
            project: "dotnet/Api.csproj".to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            test: false,
        }],
        ..Default::default()
    };
    let edges = collect_dotnet_edges(
        &root,
        &files,
        Some(&options),
        None,
        None,
        &PathInterner::default(),
    );
    assert!(edges.iter().all(|(_, _, kind)| *kind != EdgeKind::HttpCall));
}

#[test]
fn http_target_globs_apply_to_routes_but_not_to_clients() {
    let root = lang_fixture("http-clients");
    let caller = root.join("python/client.py");
    let included = root.join("go/routes.go");
    let excluded = root.join("dotnet/Routes.cs");
    let calls = ["/users".to_string()];
    let routes = [("/users".to_string(), "ListUsers".to_string())];
    let files = [
        HttpClientFile {
            path: &caller,
            calls: &calls,
            routes: &[],
        },
        HttpClientFile {
            path: &included,
            calls: &[],
            routes: &routes,
        },
        HttpClientFile {
            path: &excluded,
            calls: &[],
            routes: &routes,
        },
    ];
    let options = GraphConfigOptions {
        project_route_globset: compile_project_route_globset(&["go/**".to_string()]),
        ..Default::default()
    };
    let interner = PathInterner::default();
    assert_eq!(
        http_edges_for_client_files(&files, &interner, |path| {
            route_file_allowed(&root, path, &options)
        }),
        [(
            NodeId::file_in(&interner, &caller),
            NodeId::file_in(&interner, &included),
            EdgeKind::HttpCall,
        )]
    );
}
