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

#[test]
fn invalid_explicit_setup_policies_fail_planning_and_impact_without_a_rule() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/explicit-policy-setups");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    for (name, expected) in [
        ("invalid", "invalid repository-relative path"),
        ("empty", "invalid repository-relative path"),
        ("absolute", "invalid repository-relative path"),
        ("no-include", "requires a nonempty include matcher"),
    ] {
        let config = root.join(format!(".no-mistakes-{name}.yml"));
        let mut args = vitest_setup_args(root.clone(), vec![root.join("setup/web.ts")]);
        args.config = Some(config.clone());
        let error = crate::tests::plan::generate_plan(&args).unwrap_err();
        assert!(error.to_string().contains(expected), "{name}: {error}");
        let error = crate::tests::impact::generate_impact_plan(&crate::tests::ImpactArgs {
            entrypoints: vec!["setup/web.ts".to_string()],
            entrypoint_symbols: Vec::new(),
            include_symbols: false,
            root: root.clone(),
            config: Some(config),
            tsconfig: None,
            format: None,
            json: true,
        })
        .unwrap_err();
        assert!(error.to_string().contains(expected), "{name}: {error}");
    }
}

#[test]
fn explicit_setup_triggers_use_the_effective_policy_configuration() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/explicit-policy-setups");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    for (file, expected) in [
        (".no-mistakes.yml", Vec::<&str>::new()),
        ("custom.yml", vec!["web/live.test.ts"]),
    ] {
        let mut args = vitest_setup_args(root.clone(), vec![root.join(file)]);
        args.config = Some(root.join("custom.yml"));
        let plan = crate::tests::plan::generate_plan(&args).unwrap();
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            expected,
            "{file}: {plan:#?}"
        );
        if expected.is_empty() {
            assert!(!plan.fallback_triggered);
        }
        let impact = crate::tests::impact::generate_impact_plan(&crate::tests::ImpactArgs {
            entrypoints: vec![file.to_string()],
            entrypoint_symbols: Vec::new(),
            include_symbols: false,
            root: root.clone(),
            config: Some(root.join("custom.yml")),
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
            expected,
            "{file}: {impact:#?}"
        );
    }
}
