#[test]
fn tests_plan_json_direct_test_owner_uses_framework_ownership_without_plan_policy() {
    let output = tests_plan_json_impl(
        json!({
            "root": fixture_root("test-plan-config"),
            "framework": "vitest",
            "changedFiles": ["source.ts"],
            "environment": "all",
            "directTestOwner": true,
        })
        .to_string(),
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
        json!({
            "root": fixture_root("test-plan-config"),
            "directTestOwner": true,
        })
        .to_string(),
    )
    .unwrap_err();
    assert!(missing_framework
        .to_string()
        .contains("directTestOwner requires framework"));

    let limit = tests_plan_json_impl(
        json!({
            "root": fixture_root("test-plan-config"),
            "framework": "vitest",
            "directTestOwner": true,
            "limitFiles": 1,
        })
        .to_string(),
    )
    .unwrap_err();
    assert!(limit.to_string().contains("directTestOwner conflicts"));
}

#[test]
fn tests_plan_json_direct_test_owner_reports_changed_resource_diagnostics() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/test-plan/resource-impact");
    let output = tests_plan_json_impl(
        json!({
            "root": root,
            "framework": "vitest",
            "changedFiles": ["extractor-dynamic.ts"],
            "directTestOwner": true,
        })
        .to_string(),
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
