use super::*;

#[test]
fn explicit_policy_setup_and_helper_select_only_matching_project_tests() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/explicit-policy-setups");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    for file in ["setup/web.ts", "setup/helper.ts"] {
        let args = vitest_setup_args(root.clone(), vec![root.join(file)]);
        let plan = crate::tests::plan::generate_plan(&args).unwrap();
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["web/live.test.ts"],
            "{file}: {plan:#?}"
        );
        assert!(!plan.fallback_triggered, "{plan:#?}");
        assert!(
            plan.selected_tests[0]
                .reasons
                .iter()
                .any(|reason| reason.via.iter().any(|kind| kind == "vitest-setup")),
            "{plan:#?}"
        );
        let impact = crate::tests::impact::generate_impact_plan(&crate::tests::ImpactArgs {
            entrypoints: vec![file.to_string()],
            entrypoint_symbols: Vec::new(),
            include_symbols: false,
            root: root.clone(),
            config: None,
            tsconfig: None,
            format: None,
            json: true,
        })
        .unwrap();
        assert_eq!(
            impact
                .selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["web/live.test.ts"],
            "{file}: {impact:#?}"
        );
    }
}
