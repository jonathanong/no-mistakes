#[test]
fn configured_language_http_clients_match_route_files() {
    let root = lang_fixture("http-clients");
    let cases = [
        (
            "python/routes.py",
            "python/client.py",
            "python/computed.py",
            "/users",
        ),
        ("go/routes.go", "go/client.go", "go/computed.go", "/users"),
        (
            "rust/src/routes.rs",
            "rust/src/client.rs",
            "rust/src/computed.rs",
            "/users",
        ),
        (
            "rails/config/routes.rb",
            "rails/client.rb",
            "rails/computed.rb",
            "/users",
        ),
        (
            "php/routes/web.php",
            "php/client.php",
            "php/computed.php",
            "/users",
        ),
        (
            "java/src/main/java/com/example/Users.java",
            "java/src/main/java/com/example/Client.java",
            "java/src/main/java/com/example/Computed.java",
            "/api/users",
        ),
        (
            "kotlin/src/main/kotlin/com/example/Users.kt",
            "kotlin/src/main/kotlin/com/example/Client.kt",
            "kotlin/src/main/kotlin/com/example/Computed.kt",
            "/api/users",
        ),
        (
            "elixir/lib/router.ex",
            "elixir/lib/client.ex",
            "elixir/lib/computed.ex",
            "/users",
        ),
        (
            "dotnet/Routes.cs",
            "dotnet/Client.cs",
            "dotnet/Computed.cs",
            "/users",
        ),
    ];
    for (route, client, computed, _) in cases {
        let output = crate::napi_api::dependents_json_impl(
            crate::napi_api::options::test_json_arg(serde_json::json!({
                "root": &root,
                "files": [route],
                "relationships": ["http"]
            })),
        )
        .unwrap_or_else(|error| panic!("{route}: {error}"));
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        let files: Vec<String> = value["files"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|file| file["path"].as_str().map(|path| path.replace('\\', "/")))
            .collect();
        assert!(
            files.iter().any(|path| path == client),
            "{route} dependents {files:?} missing {client}"
        );
        assert!(
            files.iter().all(|path| path != computed),
            "{route} dependents include unsupported URL source {computed}: {files:?}"
        );
    }
    let output = crate::napi_api::server_route_edges_json_impl(
        crate::napi_api::options::test_json_arg(serde_json::json!({ "root": &root })),
    )
    .unwrap_or_else(|error| panic!("server edges: {error}"));
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let edges = value.as_array().unwrap();
    for (_, client, computed, pattern) in cases {
        assert!(
            edges.iter().any(|edge| {
                edge["kind"] == "client-call"
                    && edge["to"] == pattern
                    && edge["from"].as_str().unwrap_or("").replace('\\', "/") == client
            }),
            "missing client-call {client} -> {pattern}: {edges:?}"
        );
        assert!(
            edges
                .iter()
                .all(|edge| { edge["from"].as_str().unwrap_or("").replace('\\', "/") != computed }),
            "unsupported URL source {computed} emitted an edge: {edges:?}"
        );
    }
}

#[test]
fn cross_language_and_parameterized_client_routes_keep_their_report_patterns() {
    let root = lang_fixture("http-clients");
    for route in ["go/routes.go", "dotnet/Routes.cs"] {
        let report: serde_json::Value = serde_json::from_str(
            &crate::napi_api::dependents_json_impl(serde_json::json!({
                "root": root,
                "files": [route],
                "relationships": ["http"]
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(report["files"].as_array().unwrap().iter().any(|file| {
            file["path"].as_str().is_some_and(|path| path.replace('\\', "/") == "python/client.py")
        }), "{route}: {report}");
    }
    let edges: Vec<serde_json::Value> = serde_json::from_str(
        &crate::napi_api::server_route_edges_json_impl(serde_json::json!({"root": root})).unwrap(),
    )
    .unwrap();
    for route in ["/shared/client-check", "/customers/{customer_id}", "/members/{id:int}"] {
        assert!(edges.iter().any(|edge| {
            edge["kind"] == "client-call"
                && edge["to"] == route
                && edge["from"].as_str().is_some_and(|path| path.replace('\\', "/") == "python/client.py")
        }), "{route}: {edges:?}");
    }
}

#[test]
fn server_edge_filters_exclude_unselected_language_client_sources() {
    let root = lang_fixture("http-clients");
    let edges: Vec<serde_json::Value> = serde_json::from_str(
        &crate::napi_api::server_route_edges_json_impl(serde_json::json!({
            "root": root,
            "filters": ["go/routes.go", "go/client.go"]
        }))
        .unwrap(),
    )
    .unwrap();
    let clients: Vec<_> = edges.iter()
        .filter(|edge| edge["kind"] == "client-call")
        .map(|edge| edge["from"].as_str().unwrap().replace('\\', "/"))
        .collect();
    assert_eq!(clients, ["go/client.go"], "{edges:?}");
}

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
