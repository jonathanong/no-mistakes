#[test]
fn analyze_project_dispatches_flow_report() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "reports": [{
                "type": "flow",
                "id": "flow",
                "target": "utils.mts#parseDate",
                "direction": "both",
                "depth": 1,
                "relationships": ["import"]
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();

    assert_eq!(value["reports"][0]["id"], "flow");
    assert_eq!(
        value["reports"][0]["result"]["target"],
        "utils.mts#parseDate"
    );
    assert!(!value["reports"][0]["result"]["nodes"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn flow_napi_rejects_unknown_direction() {
    let error = crate::napi_api::flow_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "target": "utils.mts#parseDate",
            "direction": "sideways"
        })
        .to_string(),
    )
    .unwrap_err();

    assert!(error.reason.contains("unknown flow direction: sideways"));
}

#[test]
fn flow_napi_direct_impl_returns_report_and_validates_options() {
    let output = crate::napi_api::flow_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "target": "utils.mts#parseDate",
            "direction": "deps",
            "depth": 1,
            "relationships": ["import"]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["target"], "utils.mts#parseDate");

    let missing_target = crate::napi_api::flow_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol")
        })
        .to_string(),
    )
    .unwrap_err();
    assert!(missing_target
        .reason
        .contains("target is required for flow"));

    let bad_relationship = crate::napi_api::flow_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "target": "utils.mts#parseDate",
            "relationships": ["missing"]
        })
        .to_string(),
    )
    .unwrap_err();
    assert!(bad_relationship.reason.contains("unknown relationship"));
}

#[test]
fn flow_napi_direct_impl_includes_vitest_setup_relationships() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/vitest-setup-dependencies");
    let output = crate::napi_api::flow_json_impl(
        json!({
            "root": root,
            "target": "setup/conditional-a.ts",
            "direction": "dependents",
            "depth": 1,
            "relationships": ["test"]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    assert!(value["edges"].as_array().unwrap().iter().any(|edge| {
        edge["from"] == "conditional-owner/conditional.test.ts"
            && edge["to"] == "setup/conditional-a.ts"
            && edge["kind"] == "vitest-setup"
    }), "{value:#}");
}
