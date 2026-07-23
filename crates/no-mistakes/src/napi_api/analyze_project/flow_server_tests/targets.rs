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
        json!({ "root": root, "framework": "vitest", "files": ["tests/unit.test.ts"] })
            .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let target = &value["tests"][0]["targets"][0];

    assert_eq!(target["workspace"], true);
    assert_eq!(target["config"], "vitest.projects.ts");
    assert_eq!(target["runner_args"], json!([
        "--workspace", "vitest.projects.ts", "--project", "unit", "tests/unit.test.ts"
    ]));
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
    let targets = value["tests"].as_array().unwrap().iter()
        .flat_map(|test| test["targets"].as_array().unwrap()).collect::<Vec<_>>();
    let json_target = targets.iter().find(|target| target["project"] == "json-inline").unwrap();
    let string_target = targets.iter().find(|target| target["project"] == "json-string").unwrap();
    assert_eq!(json_target["config"], "vitest.workspace.json");
    assert_eq!(string_target["config"], "vitest.projects.json");
    assert_eq!(json_target["runner_args"], json!([
        "--workspace", "vitest.workspace.json", "--project", "json-inline", "inline/inline.test.ts"
    ]));
    for target in targets {
        assert_eq!(target["workspace"], true);
        assert_eq!(target["runner_args"][0], "--workspace");
    }
}

#[test]
fn tests_targets_napi_rejects_missing_files() {
    for options in [
        json!({ "root": fixture_root("test-plan-project-discovery"), "framework": "vitest" }),
        json!({ "root": fixture_root("test-plan-project-discovery"), "framework": "vitest", "files": [] }),
    ] {
        let error = crate::napi_api::cli_parity::tests_targets_json_impl(options.to_string())
            .expect_err("missing files should fail");
        assert!(error.reason.contains("files is required"));
    }
}
