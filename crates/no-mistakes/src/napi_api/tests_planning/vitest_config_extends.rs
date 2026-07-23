#[test]
fn tests_plan_napi_traces_static_vitest_config_extends() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-extends-config");
    let output = tests_plan_json_impl(
        json!({
            "framework": "vitest",
            "root": root,
            "changedFiles": ["base-setup.ts"],
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(
        plan["selected_tests"][0]["test_file"],
        "extended/owned.test.ts",
        "{plan:#}"
    );
    assert_eq!(plan["fallback_triggered"], false, "{plan:#}");
}
