#[test]
fn tests_impact_json_traverses_prepared_vitest_setup_edges() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    );
    let options = json!({
        "root": root,
        "config": root.join("resolved.no-mistakes.yml"),
        "entrypoints": ["setup/resolved-helper.ts"]
    })
    .to_string();
    let output = tests_impact_json_impl(options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected.iter().any(|test| test["test_file"] == "resolved/resolved.test.ts"),
        "prepared Vitest setup edge should reach its owner: {selected:?}"
    );

    let output = tests_impact_json_impl(
        json!({
            "root": root,
            "entrypoints": ["shared-setup/named-member-star.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 1, "{selected:?}");
    assert_eq!(
        selected[0]["test_file"],
        "named-member-owner/named-member.test.ts"
    );

    let output = tests_impact_json_impl(
        json!({
            "root": root,
            "entrypoints": ["inherits/setup/missing.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 1, "{selected:?}");
    assert_eq!(selected[0]["test_file"], "inherits/inherited.test.ts");
    assert!(selected[0]["reasons"].as_array().unwrap().iter().any(|reason| {
        reason["via"]
            .as_array()
            .is_some_and(|via| via.last().is_some_and(|edge| edge == "vitest-setup"))
    }), "{selected:#?}");
}
