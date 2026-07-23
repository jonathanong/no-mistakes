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

    let output = tests_impact_json_impl(
        json!({
            "root": root,
            "entrypoints": ["arbitrary-project-match/setup/arbitrary.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(
        plan["selected_tests"][0]["test_file"],
        "arbitrary-project-match/arbitrary.fixture",
        "{plan:#}"
    );

    let output = tests_impact_json_impl(
        json!({ "root": root, "entrypoints": ["config/setup-selector.ts"] }).to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(plan["fallback_triggered"], true, "{plan:#}");
    assert_eq!(plan["selected_tests"][0]["test_file"], "inherits/inherited.test.ts");

    let output = tests_impact_json_impl(
        json!({
            "root": root,
            "entrypoints": ["runtime-owner/setup/deleted-runtime-helper.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(plan["fallback_triggered"], true, "{plan:#}");
    assert_eq!(
        plan["selected_tests"][0]["test_file"],
        "runtime-owner/runtime-owner.test.ts",
        "{plan:#}"
    );
    assert!(plan["fallback_reason"]
        .as_str()
        .is_some_and(|reason| reason.contains("transitive dependency of a resolved setup was deleted")), "{plan:#}");

    let output = tests_impact_json_impl(
        json!({ "root": root, "entrypoints": ["configless-project/default.test.ts"] })
            .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(
        plan["selected_tests"][0]["test_file"],
        "configless-project/default.test.ts",
        "{plan:#}"
    );
}

#[test]
fn tests_impact_json_keeps_native_tests_when_optional_vitest_is_invalid() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/impact-invalid-vitest-config"),
    );
    let output = tests_impact_json_impl(
        json!({ "root": root, "entrypoints": ["tests/ServiceTests.cs"] }).to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["selected_tests"][0]["test_file"], "tests/ServiceTests.cs");
    assert_eq!(plan["selected_tests"][0]["reasons"][0]["via"][0], "self");
}

#[test]
fn tests_impact_json_rejects_valid_vitest_discovery_errors() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/impact-invalid-vitest-discovery"),
    );
    let result = tests_impact_json_impl(
        json!({ "root": root, "entrypoints": ["tests/ServiceTests.cs"] }).to_string(),
    );

    assert!(result.is_err());
}
