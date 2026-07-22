#[test]
fn analyze_project_dispatches_server_contracts_report() {
    let output = analyze_project_json_impl(
        json!({
            "root": server_fixture_root("express"),
            "reports": [{ "type": "serverContracts", "id": "contracts" }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();

    assert_eq!(value["reports"][0]["id"], "contracts");
    assert!(value["reports"][0]["result"]["mismatches"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["missingParams"] == json!(["unused"])));
}

#[test]
fn analyze_project_server_routes_and_contracts_share_union_facts() {
    let source = parser_count_server_fixture();
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    crate::ast::begin_parse_count(&root);

    let output = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [
                { "type": "serverRoutes", "id": "routes" },
                { "type": "serverContracts", "id": "contracts" }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let value: Value = serde_json::from_str(&output).unwrap();

    assert_eq!(
        value["reports"][0]["result"]["routes"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        value["reports"][1]["result"]["clientRefs"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let files = crate::server_routes::prepare_analysis(&root, None)
        .unwrap()
        .source_files;
    assert_eq!(counts.len(), files.len(), "{counts:#?}");
    assert!(
        files.iter().all(|file| counts.get(file) == Some(&1)),
        "combined server reports must parse each source once: {counts:#?}"
    );
}

#[test]
fn server_contracts_napi_direct_impl_returns_report() {
    let output = crate::napi_api::server_contracts_json_impl(
        json!({
            "root": server_fixture_root("express")
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();

    assert!(value["routes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["route"] == "/api/v1/search"));

    let route_list = crate::napi_api::server_route_list_json_impl(
        json!({
            "root": server_fixture_root("express"),
            "files": ["/api/v1/search"]
        })
        .to_string(),
    )
    .unwrap();
    let routes: Value = serde_json::from_str(&route_list).unwrap();
    assert_eq!(routes.as_array().unwrap()[0]["route"], "/api/v1/search");
}

#[test]
fn server_contracts_napi_honors_roots_scope() {
    let output = crate::napi_api::server_contracts_json_impl(
        json!({
            "root": server_fixture_root("express"),
            "roots": ["backend/api/users.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();

    assert!(value["routes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["route"] == "/api/v1/search"));
    assert!(value["clientRefs"].as_array().unwrap().is_empty());
}

#[test]
fn tests_targets_napi_reports_project_commands() {
    let output = crate::napi_api::cli_parity::tests_targets_json_impl(
        json!({
            "root": fixture_root("test-plan-project-discovery"),
            "framework": "vitest",
            "files": ["web/storybook/button.stories.tsx"]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let targets = value["tests"][0]["targets"].as_array().unwrap();

    assert!(targets.iter().any(|target| target["project"] == "browser"));
    assert!(targets.iter().any(|target| target["project"] == "stories"));
}

#[test]
fn tests_targets_napi_preserves_vitest_workspace_source() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-projects-target");
    let output = crate::napi_api::cli_parity::tests_targets_json_impl(
        json!({
            "root": root,
            "framework": "vitest",
            "files": ["tests/unit.test.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let target = &value["tests"][0]["targets"][0];

    assert_eq!(target["workspace"], true);
    assert_eq!(target["config"], "vitest.projects.ts");
    assert_eq!(
        target["runner_args"],
        json!([
            "--workspace",
            "vitest.projects.ts",
            "--project",
            "unit",
            "tests/unit.test.ts"
        ])
    );
}

#[test]
fn tests_targets_napi_preserves_json_workspace_sources() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-workspace-json");
    let output = crate::napi_api::cli_parity::tests_targets_json_impl(
        json!({
            "root": root,
            "framework": "vitest",
            "files": ["inline/inline.test.ts", "string-project/string.test.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let targets = value["tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| &test["targets"][0])
        .collect::<Vec<_>>();

    assert_eq!(targets[0]["config"], "vitest.workspace.json");
    assert_eq!(targets[1]["config"], "vitest.projects.json");
    for target in targets {
        assert_eq!(target["workspace"], true);
        assert_eq!(target["runner_args"][0], "--workspace");
    }
}

#[test]
fn tests_targets_napi_rejects_missing_files() {
    for options in [
        json!({
            "root": fixture_root("test-plan-project-discovery"),
            "framework": "vitest"
        }),
        json!({
            "root": fixture_root("test-plan-project-discovery"),
            "framework": "vitest",
            "files": []
        }),
    ] {
        let error = crate::napi_api::cli_parity::tests_targets_json_impl(options.to_string())
            .expect_err("missing files should fail");
        assert!(error.reason.contains("files is required"));
    }
}
