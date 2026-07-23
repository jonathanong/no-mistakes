use super::*;
use no_mistakes::codebase::dependencies::graph::NodeId;

fn framework_args(root: &Path, framework: TestFramework) -> PlanArgs {
    PlanArgs {
        framework: Some(framework),
        root: root.to_path_buf(),
        config: None,
        tsconfig: None,
        base: None,
        head: None,
        from_git_diff: None,
        changed_file: vec![root.join("src/unit.ts")],
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
        json: false,
    }
}

#[test]
fn non_framework_plan_reuses_prepared_runner_config_facts_and_filter() {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/non-framework-prepared-plan"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let mut args = framework_args(&root, TestFramework::Vitest);
    args.framework = None;
    crate::ast::begin_parse_count(&root);
    let plan = crate::tests::plan::generate_plan(&args).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    assert!(plan
        .selected_tests
        .iter()
        .any(|test| test.test_file == "src/unit.test.ts"));
    assert_eq!(counts.len(), 6, "{counts:#?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
}

#[test]
fn complete_prepared_graph_keeps_standard_skipped_playwright_sources_outside_its_universe() {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check-discovery/project-pattern-reopen/fixture"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let changed = root.join("web/next.config.ts");
    let mut args = framework_args(&root, TestFramework::Vitest);
    args.changed_file = vec![changed.clone()];
    let prepared = PreparedTestPlanRequest::prepare(&args).unwrap();
    crate::ast::begin_parse_count(&root);
    let graph = prepared.graph().unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    assert!(graph.dependencies_of_node(&NodeId::File(changed)).is_some());
    assert!(graph
        .dependencies_of_node(&NodeId::File(root.join("web/fixtures/included.ts")))
        .is_none());
    assert!(!counts.contains_key(&root.join("web/fixtures/included.ts")));
}

#[test]
fn framework_plan_leaves_invalid_unrequested_runner_untouched() {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/framework-demand-invalid-unrequested"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let args = framework_args(&root, TestFramework::Vitest);
    crate::ast::begin_parse_count(&root);
    let prepared = PreparedTestPlanRequest::prepare(&args).unwrap();
    let discovered = prepared.discover_tests(TestFramework::Vitest).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    assert!(discovered.tests.contains(&root.join("src/unit.test.ts")));
    assert_eq!(counts.get(&root.join("vitest.config.ts")), Some(&1));
    // Vitest fallback ownership depends on Playwright projects. A malformed
    // Playwright config is prepared once but remains a lossy ownership input,
    // so it does not fail the requested Vitest discovery.
    assert_eq!(counts.get(&root.join("playwright.config.ts")), Some(&1));
}

#[test]
fn requested_runner_projects_reuses_the_prepared_vitest_catalog() {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/framework-demand-invalid-unrequested"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let args = framework_args(&root, TestFramework::Vitest);
    let prepared = PreparedTestPlanRequest::prepare(&args).unwrap();

    let projects = prepared
        .requested_runner_projects(TestRunner::Vitest)
        .unwrap();

    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].config.as_deref(), Some("vitest.config.ts"));
    assert!(prepared
        .requested_runner_projects(TestRunner::Playwright)
        .is_err());
}

#[test]
fn prepared_vitest_setup_projects_retain_visible_test_candidates() {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let mut args = framework_args(&root, TestFramework::Vitest);
    args.changed_file = vec![root.join("setup/root.ts")];
    let prepared = PreparedTestPlanRequest::prepare(&args).unwrap();

    let projects = prepared.prepared_test_projects.vitest_setup_projects();
    let arbitrary_project = projects
        .iter()
        .find(|project| {
            project
                .setups
                .iter()
                .any(|(path, _)| path == &root.join("arbitrary-project-match/setup/arbitrary.ts"))
        })
        .expect("explicit project retains its static setup file");
    assert!(arbitrary_project
        .tests
        .contains(&root.join("arbitrary-project-match/arbitrary.fixture")));
}

#[test]
fn requested_runner_failure_is_memoized() {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/framework-demand-invalid-unrequested"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let args = framework_args(&root, TestFramework::Playwright);
    crate::ast::begin_parse_count(&root);
    let prepared = PreparedTestPlanRequest::prepare(&args).unwrap();
    let first = prepared
        .discover_tests(TestFramework::Playwright)
        .unwrap_err();
    let second = prepared
        .discover_tests(TestFramework::Playwright)
        .unwrap_err();
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(first.to_string(), second.to_string());
    assert_eq!(prepared.framework_discovery_count(), 1);
    assert_eq!(counts.get(&root.join("playwright.config.ts")), Some(&1));
    assert!(!counts.contains_key(&root.join("vitest.config.ts")));
}

#[test]
fn native_framework_plans_reuse_discovery_facts_in_the_graph() {
    let dotnet_source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/dotnet-test-plan/fixture"),
    );
    let dotnet_fixture = crate::test_support::materialize_saved_fixture(&dotnet_source);
    let dotnet_root = dotnet_fixture.path().canonicalize().unwrap();
    let mut dotnet_args = framework_args(&dotnet_root, TestFramework::Dotnet);
    dotnet_args.changed_file = vec![dotnet_root.join("dotnet-clients/src/App/FeedClient.cs")];
    no_mistakes::codebase::dotnet::test_support::begin_fact_collection_count(&dotnet_root);
    let dotnet = PreparedTestPlanRequest::prepare(&dotnet_args).unwrap();
    dotnet.graph().unwrap();
    assert_eq!(
        no_mistakes::codebase::dotnet::test_support::finish_fact_collection_count(&dotnet_root),
        1
    );

    let swift_source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/swift-test-plan/fixture"),
    );
    let swift_fixture = crate::test_support::materialize_saved_fixture(&swift_source);
    let swift_root = swift_fixture.path().canonicalize().unwrap();
    let mut swift_args = framework_args(&swift_root, TestFramework::Swift);
    swift_args.changed_file = vec![swift_root.join("backend/api/feeds.mts")];
    no_mistakes::codebase::swift::test_support::begin_fact_collection_count(&swift_root);
    let swift = PreparedTestPlanRequest::prepare(&swift_args).unwrap();
    swift.graph().unwrap();
    assert_eq!(
        no_mistakes::codebase::swift::test_support::finish_fact_collection_count(&swift_root),
        1
    );
}
