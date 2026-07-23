#[test]
fn tests_targets_napi_prefers_default_vitest_workspace_ownership() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-workspace-precedence");
    let output = crate::napi_api::cli_parity::tests_targets_json_impl(
        json!({
            "root": root,
            "framework": "vitest",
            "files": ["workspace/owned.test.ts"],
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let target = &value["tests"][0]["targets"][0];

    assert_eq!(target["config"], "vitest.workspace.ts");
    assert_eq!(target["workspace"], true);
    assert_eq!(target["project"], "workspace");

    let root_only = crate::napi_api::cli_parity::tests_targets_json_impl(
        json!({
            "root": root,
            "framework": "vitest",
            "files": ["root/root.test.ts"],
        })
        .to_string(),
    )
    .unwrap();
    let root_only: Value = serde_json::from_str(&root_only).unwrap();
    let root_target = &root_only["tests"][0]["targets"][0];
    assert_eq!(root_target["config"], Value::Null);
    assert_eq!(root_target["project"], Value::Null);
}
