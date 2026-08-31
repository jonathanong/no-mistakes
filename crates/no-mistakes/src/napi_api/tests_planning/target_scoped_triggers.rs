#[test]
fn tests_plan_json_exposes_target_scoped_configured_triggers() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/target-scoped-triggers");
    let output = tests_plan_json_impl(
        crate::napi_api::options::test_json_arg(serde_json::json!({
            "framework": "vitest",
            "root": root,
            "changedFiles": ["migrations/001.sql"]
        })
        .to_string(),)
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert_eq!(plan["selected_tests"][0]["test_file"], "src/db/db.test.ts");
    assert_eq!(
        plan["selected_tests"][0]["reasons"][0]["via"],
        serde_json::json!(["configured-trigger"])
    );
    assert_eq!(
        plan["selected_tests"][0]["targets"][0]["project"],
        "database"
    );

    let changed_test_output = tests_plan_json_impl(
        crate::napi_api::options::test_json_arg(
            serde_json::json!({
                "framework": "vitest",
                "root": root,
                "config": root.join("configs/changed-tests-default.yml"),
                "changedFiles": ["src/web/web.test.ts"]
            })
            .to_string(),
        ),
    )
    .unwrap();
    let changed_test_plan: serde_json::Value =
        serde_json::from_str(&changed_test_output).unwrap();
    assert_eq!(
        changed_test_plan["selected_tests"]
            .as_array()
            .unwrap()
            .iter()
            .map(|test| test["test_file"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["src/web/policy.test.ts", "src/web/web.test.ts"]
    );

    let opted_in_output = tests_plan_json_impl(
        crate::napi_api::options::test_json_arg(
            serde_json::json!({
                "framework": "vitest",
                "root": root,
                "config": root.join("configs/include-changed-tests.yml"),
                "changedFiles": ["src/web/web.test.ts"]
            })
            .to_string(),
        ),
    )
    .unwrap();
    let opted_in_plan: serde_json::Value = serde_json::from_str(&opted_in_output).unwrap();
    assert_eq!(
        opted_in_plan["selected_tests"]
            .as_array()
            .unwrap()
            .iter()
            .map(|test| test["test_file"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "src/shared.test.ts",
            "src/web/policy.test.ts",
            "src/web/web.test.ts"
        ]
    );
}
