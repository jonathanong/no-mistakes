#[test]
fn tests_plan_json_direct_test_owner_uses_framework_ownership_without_plan_policy() {
    let output = tests_plan_json_impl(
        crate::napi_api::options::test_json_arg(json!({
            "root": fixture_root("test-plan-config"),
            "framework": "vitest",
            "changedFiles": ["source.ts"],
            "environment": "all",
            "directTestOwner": true,
        })
        .to_string(),)
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(plan["groups"][0]["type"], "direct-test-owner");
    assert_eq!(plan["selected_tests"][0]["test_file"], "source.test.mts");
    assert_eq!(
        plan["selected_tests"][0]["reasons"][0]["path"],
        json!(["source.ts", "source.test.mts"])
    );
    assert_eq!(plan["selected_tests"][0]["targets"][0]["runner"], "vitest");
}

#[test]
fn tests_plan_json_direct_test_owner_requires_framework_and_rejects_policy_overrides() {
    let missing_framework = tests_plan_json_impl(
        crate::napi_api::options::test_json_arg(json!({
            "root": fixture_root("test-plan-config"),
            "directTestOwner": true,
        })
        .to_string(),)
    )
    .unwrap_err();
    let missing_framework = missing_framework.to_string();
    assert!(missing_framework.contains("directTestOwner requires framework"));
    assert!(missing_framework.contains("framework: \"vitest\""));

    let limit = tests_plan_json_impl(
        crate::napi_api::options::test_json_arg(json!({
            "root": fixture_root("test-plan-config"),
            "framework": "vitest",
            "directTestOwner": true,
            "limitFiles": 1,
        })
        .to_string(),)
    )
    .unwrap_err();
    let limit = limit.to_string();
    assert!(limit.contains("directTestOwner conflicts"));
    assert!(limit.contains("remove those policy overrides"));

    let entrypoints = tests_plan_json_impl(
        crate::napi_api::options::test_json_arg(json!({
            "root": fixture_root("test-plan-config"),
            "framework": "vitest",
            "directTestOwner": true,
            "entrypoints": ["source.ts"],
        })
        .to_string(),)
    )
    .unwrap_err();
    assert!(entrypoints
        .to_string()
        .contains("directTestOwner conflicts with entrypoints"));
    assert!(entrypoints.to_string().contains("testsImpact"));
}

#[test]
fn tests_plan_json_direct_test_owner_reports_changed_resource_diagnostics() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/test-plan/resource-impact");
    let output = tests_plan_json_impl(
        crate::napi_api::options::test_json_arg(json!({
            "root": root,
            "framework": "vitest",
            "changedFiles": ["extractor-dynamic.ts"],
            "directTestOwner": true,
        })
        .to_string(),)
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["groups"][0]["type"], "direct-test-owner");
    assert_eq!(
        plan["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|warning| {
                (
                    warning["type"].as_str(),
                    warning["file"].as_str(),
                    warning["line"].as_u64(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (
                Some("dynamic-resource-path"),
                Some("extractor-dynamic.ts"),
                Some(4),
            ),
            (
                Some("dynamic-resource-cwd"),
                Some("extractor-dynamic.ts"),
                Some(6),
            ),
        ]
    );
}
