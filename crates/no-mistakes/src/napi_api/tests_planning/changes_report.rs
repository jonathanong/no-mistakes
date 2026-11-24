#[test]
fn tests_plan_json_routes_changes_reports() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/tests-plan-changes");
    let output = tests_plan_json_impl(json!({
        "framework": "vitest",
        "root": root,
        "changes": "reports/matched.json",
        "changedFiles": ["data/example.json"]
    }))
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(plan["fallback_triggered"], false, "{plan:#}");
    assert_eq!(
        plan["selected_tests"].as_array().unwrap().len(),
        1,
        "{plan:#}"
    );
    assert_eq!(
        plan["selected_tests"][0]["test_file"],
        "tests/reader.test.ts"
    );
    assert_eq!(plan["selected_tests"][0]["targets"][0]["project"], "reader");

    let invalid = tests_plan_json_impl(json!({
        "framework": "vitest",
        "root": root,
        "changes": "reports/matched.json",
        "directTestOwner": true
    }))
    .unwrap_err();
    assert!(invalid
        .reason
        .contains("directTestOwner conflicts with changes"));
}
