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
