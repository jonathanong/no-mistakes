use serde_json::json;

#[test]
fn tests_plan_json_samples_limited_group_from_config() {
    let root = super::fixture_root("test-plan-sample-when-limited");
    let options = json!({
        "framework": "vitest",
        "root": root,
        "changedFiles": ["changed.test.mts"],
        "environment": "sampledSnake"
    })
    .to_string();
    let output = crate::napi_api::tests_plan_json_impl(options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 2);
    assert_eq!(plan["groups"][1]["selected"][0], "zeta.test.mts");
}
