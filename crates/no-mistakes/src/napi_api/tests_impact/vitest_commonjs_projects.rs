#[test]
fn tests_impact_json_keeps_named_commonjs_project_setup_owner_exact() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    );
    let output = tests_impact_json_impl(
        json!({
            "root": root,
            "entrypoints": ["cjs-named-replacement-owner/setup/named-replacement.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(plan["fallback_triggered"], false, "{plan:#}");
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1, "{plan:#}");
    assert_eq!(
        plan["selected_tests"][0]["test_file"],
        "cjs-named-replacement-owner/cjs-named-replacement.test.ts",
        "{plan:#}"
    );
}

#[test]
fn tests_impact_json_ignores_named_commonjs_excluded_projects() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    );
    let output = tests_impact_json_impl(
        json!({
            "root": root,
            "entrypoints": ["cjs-commonjs-excluded-owner/setup/named-reexport-excluded.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(plan["fallback_triggered"], false, "{plan:#}");
    assert!(plan["selected_tests"].as_array().unwrap().is_empty(), "{plan:#}");
}
