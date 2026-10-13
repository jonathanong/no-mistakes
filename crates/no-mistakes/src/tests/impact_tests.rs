use super::*;

#[test]
fn impact_classifies_missing_setup_helper_entrypoints_as_deleted() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    );
    let plan = generate_impact_plan(&ImpactArgs {
        entrypoints: vec!["runtime-owner/setup/deleted-runtime-helper.ts".to_string()],
        entrypoint_symbols: Vec::new(),
        include_symbols: false,
        root,
        config: None,
        tsconfig: None,
        format: None,
        json: true,
    })
    .unwrap();

    assert!(plan.fallback_triggered, "{plan:#?}");
    assert_eq!(
        plan.selected_tests[0].test_file,
        "runtime-owner/runtime-owner.test.ts"
    );
    assert!(plan.fallback_reason.as_deref().is_some_and(|reason| {
        reason.contains("transitive dependency of a resolved setup was deleted")
    }));
}

#[test]
fn impact_keeps_native_tests_when_optional_vitest_discovery_fails() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/impact-invalid-vitest-config"),
    );
    let plan = generate_impact_plan(&ImpactArgs {
        entrypoints: vec!["tests/ServiceTests.cs".to_string()],
        entrypoint_symbols: Vec::new(),
        include_symbols: false,
        root,
        config: None,
        tsconfig: None,
        format: None,
        json: true,
    })
    .unwrap();

    assert_eq!(plan.selected_tests[0].test_file, "tests/ServiceTests.cs");
    assert_eq!(plan.selected_tests[0].reasons[0].via, ["self"]);
}

#[test]
fn impact_rejects_valid_vitest_discovery_errors() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/impact-invalid-vitest-discovery"),
    );
    let result = generate_impact_plan(&ImpactArgs {
        entrypoints: vec!["tests/ServiceTests.cs".to_string()],
        entrypoint_symbols: Vec::new(),
        include_symbols: false,
        root,
        config: None,
        tsconfig: None,
        format: None,
        json: true,
    });

    assert!(result.is_err());
}
