use super::*;

#[test]
fn tests_impact_keeps_named_commonjs_project_setup_owner_exact() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    );
    let plan = plan_for(
        &root,
        "cjs-named-replacement-owner/setup/named-replacement.ts",
    );
    assert_eq!(plan["fallback_triggered"], false, "{plan:#}");
    assert_eq!(
        plan["selected_tests"].as_array().unwrap().len(),
        1,
        "{plan:#}"
    );
    assert_eq!(
        plan["selected_tests"][0]["test_file"],
        "cjs-named-replacement-owner/cjs-named-replacement.test.ts",
        "{plan:#}"
    );
}

#[test]
fn tests_impact_ignores_excluded_commonjs_projects() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    );
    let plan = plan_for(&root, "cjs-commonjs-excluded-owner/setup/chain-excluded.ts");
    assert_eq!(plan["fallback_triggered"], false, "{plan:#}");
    assert!(
        plan["selected_tests"].as_array().unwrap().is_empty(),
        "{plan:#}"
    );
}
