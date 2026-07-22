fn resource_fixture_root() -> tempfile::TempDir {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/resource-impact"),
    );
    crate::test_support::materialize_saved_fixture(&source)
}

fn resource_plan_args(root: &Path, changed: PathBuf) -> PlanArgs {
    PlanArgs {
        framework: None,
        root: root.to_path_buf(),
        config: None,
        tsconfig: None,
        base: None,
        head: None,
        from_git_diff: None,
        changed_file: vec![changed],
        changed_files: None,
        diff: None,
        diff_stdin: false,
        diff_command: None,
        entrypoints: Vec::new(),
        entrypoint_symbols: Vec::new(),
        include_symbols: false,
        diff_content: None,
        environment: "pre-push".to_string(),
        limit_percent: None,
        limit_files: None,
        global_config_fallback: None,
        format: None,
        json: true,
    }
}

#[test]
fn literal_resource_change_selects_importing_test_with_provenance() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    let plan = generate_plan(&resource_plan_args(
        &root,
        root.join("resources/page.txt"),
    ))
    .unwrap();

    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["impact-consumer.test.ts"]
    );
    let reason = &plan.selected_tests[0].reasons[0];
    assert_eq!(reason.via, ["resource", "dependency"]);
    let Some(Some(ImpactEdgeDetail::Resource {
        consumer_file,
        call_sites,
    })) = reason.via_details.first()
    else {
        panic!("the resource transition must retain its call-site provenance");
    };
    assert_eq!(consumer_file, "impact-consumer.ts");
    assert_eq!(
        call_sites,
        &[ResourceCallSite {
            call_kind: "read-file-sync".to_string(),
            line: 3,
        }]
    );
    assert_eq!(reason.via_details.len(), reason.via.len());
}

#[test]
fn tracked_resource_under_source_skipped_directory_selects_importing_test() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    let plan = generate_plan(&resource_plan_args(
        &root,
        root.join("fixtures/schema.sql"),
    ))
    .unwrap();

    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["skipped-resource-consumer.test.ts"]
    );
    assert_eq!(plan.selected_tests[0].reasons[0].via, ["resource", "dependency"]);
}

#[test]
fn exported_member_resources_reach_the_importing_test_without_a_local_call() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    for changed in [
        "resources/exported-object.txt",
        "resources/exported-class-expression.txt",
        "resources/exported-named-class.txt",
        "resources/exported-named-root.txt",
        "resources/exported-default-root.txt",
    ] {
        let plan = generate_plan(&resource_plan_args(&root, root.join(changed))).unwrap();
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["exported-member-consumer.test.ts"],
            "{changed} must retain its exported member scope"
        );
        let reason = &plan.selected_tests[0].reasons[0];
        assert_eq!(reason.via, ["resource", "dependency"]);
        let Some(Some(ImpactEdgeDetail::Resource {
            consumer_file,
            call_sites,
        })) = reason.via_details.first()
        else {
            panic!("{changed} must preserve resource call-site provenance");
        };
        assert_eq!(consumer_file, "exported-member-consumer.ts");
        assert_eq!(call_sites.len(), 1);
        assert_eq!(call_sites[0].call_kind, "read-file-sync");
    }
}

#[test]
fn uncalled_nested_helper_under_an_exported_member_stays_pruned() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    let plan = generate_plan(&resource_plan_args(
        &root,
        root.join("resources/exported-object-unused.txt"),
    ))
    .unwrap();

    assert!(
        plan.selected_tests.is_empty(),
        "an uncalled nested helper must not become reachable through its exported aggregate root"
    );
    assert!(!plan.fallback_triggered);
}

#[test]
fn generic_default_export_expression_resources_reach_the_importing_test() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    for changed in [
        "resources/exported-default-direct.txt",
        "resources/exported-default-wrapped.txt",
        "resources/exported-default-named-class.txt",
        "resources/exported-default-anonymous-class.txt",
    ] {
        let plan = generate_plan(&resource_plan_args(&root, root.join(changed))).unwrap();
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["exported-default-expression.test.ts"],
            "{changed} must retain its generic default-export root scope"
        );
        assert_eq!(plan.selected_tests[0].reasons[0].via[0], "resource");
    }
}

#[test]
fn uncalled_nested_callback_under_default_expression_stays_pruned() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    let plan = generate_plan(&resource_plan_args(
        &root,
        root.join("resources/exported-default-nested-unused.txt"),
    ))
    .unwrap();

    assert!(
        plan.selected_tests.is_empty(),
        "an uncalled nested callback must not inherit default-root reachability"
    );
    assert!(!plan.fallback_triggered);
}

#[test]
fn dynamic_resource_consumer_warns_without_creating_a_resource_reason() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    let plan = generate_plan(&resource_plan_args(
        &root,
        root.join("extractor-dynamic.ts"),
    ))
    .unwrap();

    assert_eq!(
        plan.warnings
            .iter()
            .map(|warning| (warning.r#type.as_str(), warning.file.as_str(), warning.line))
            .collect::<Vec<_>>(),
        [
            ("dynamic-resource-path", "extractor-dynamic.ts", Some(4)),
            ("dynamic-resource-cwd", "extractor-dynamic.ts", Some(6)),
        ]
    );
    assert!(plan
        .selected_tests
        .iter()
        .flat_map(|test| &test.reasons)
        .all(|reason| !reason.via.iter().any(|via| via == "resource")));
    assert!(!plan.fallback_triggered);
}

#[test]
fn readdir_and_nested_glob_resource_changes_select_only_their_consumers() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    for (changed, expected_test, expected_kind) in [
        (
            "migrations/003-added.sql",
            "directory-consumer.test.ts",
            "read-directory-sync",
        ),
        (
            "glob-resources/nested/leaf.txt",
            "glob-consumer.test.ts",
            "glob-sync",
        ),
    ] {
        let plan = generate_plan(&resource_plan_args(&root, root.join(changed))).unwrap();
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            [expected_test],
            "{changed} should reach only its static resource consumer"
        );
        let reason = &plan.selected_tests[0].reasons[0];
        assert_eq!(reason.via[0], "resource");
        assert_eq!(reason.via_details.len(), reason.via.len());
        let Some(Some(ImpactEdgeDetail::Resource { call_sites, .. })) = reason.via_details.first()
        else {
            panic!("{changed} must include resource provenance");
        };
        assert_eq!(call_sites[0].call_kind, expected_kind);

        // The human paths contract intentionally stays test-only even when
        // the JSON reason carries resource-edge debug data.
        assert_eq!(
            crate::tests::plan_output::render(&plan, PlanFormat::Paths, "tests plan").unwrap(),
            format!("{expected_test}\n")
        );
    }
}

#[test]
fn module_relative_file_url_resources_select_their_consumer() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    let plan = generate_plan(&resource_plan_args(&root, root.join("resources/url.txt"))).unwrap();

    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["url-consumer.test.ts"]
    );
    assert_eq!(plan.selected_tests[0].reasons[0].via[0], "resource");
}

#[test]
fn dynamic_glob_pattern_warns_without_an_edge_or_fallback() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    let plan = generate_plan(&resource_plan_args(&root, root.join("dynamic-pattern.ts"))).unwrap();

    assert_eq!(
        plan.warnings
            .iter()
            .map(|warning| (warning.r#type.as_str(), warning.file.as_str(), warning.line))
            .collect::<Vec<_>>(),
        [("dynamic-resource-pattern", "dynamic-pattern.ts", Some(4))]
    );
    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["dynamic-pattern.test.ts"]
    );
    assert!(plan.selected_tests[0].reasons[0]
        .via
        .iter()
        .all(|via| via != "resource"));
    assert!(!plan.fallback_triggered);
}

#[test]
fn configured_vitest_and_playwright_plans_keep_resource_impact_scoped_to_the_runner() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    for (framework, changed, expected_test) in [
        (
            crate::tests::TestFramework::Vitest,
            "resources/page.txt",
            "impact-consumer.test.ts",
        ),
        (
            crate::tests::TestFramework::Playwright,
            "playwright-resources/page.txt",
            "playwright-resource.pw.ts",
        ),
    ] {
        let mut args = resource_plan_args(&root, root.join(changed));
        args.framework = Some(framework);
        let plan = generate_plan(&args).unwrap();
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            [expected_test],
            "{framework:?} should apply its own discovered-test filter"
        );
        assert_eq!(plan.selected_tests[0].reasons[0].via[0], "resource");
    }
}
