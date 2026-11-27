#[test]
fn traversal_stages_graph_configuration_around_one_prepared_test_project_pass() {
    let shared = concat!(
        include_str!("../shared_traversal.rs"),
        include_str!("../shared_traversal_prepare.rs"),
        include_str!("../shared_traversal_prepare_core.rs"),
        include_str!("../../test_discovery/traversal/catalog.rs")
    );
    let shared_graph = concat!(
        include_str!("../shared_traversal_graph.rs"),
        include_str!("../shared_graph_cache.rs"),
    );
    let standalone = include_str!("../../test_discovery/traversal/globs.rs");

    assert!(!shared.contains("AnalysisDataset::new"));
    assert_eq!(shared.matches("session.dataset(&root)").count(), 2);
    assert_eq!(
        shared
            .matches("session.config(&root, config_path)?")
            .count(),
        1
    );
    assert_eq!(
        shared
            .matches(".tsconfig(&root, tsconfig_path)")
            .count(),
        1
    );
    assert_eq!(shared.matches("config_from_loaded_v2(").count(), 1);
    assert_eq!(
        shared
            .matches("prepare_graph_config_with_test_filter_and_workspace(")
            .count(),
        2
    );
    assert_eq!(shared.matches("TestFileFilter::fallback_only()").count(), 1);
    assert_eq!(
        shared
            .matches("prepare_test_projects_from_visible_with_sources_and_plan(")
            .count(),
        1
    );
    assert_eq!(
        shared
            .matches("TestFileFilter::from_prepared_projects(")
            .count(),
        1
    );
    assert_eq!(
        shared
            .matches("ts_fact_plan_and_context_for_plan_with_prepared(")
            .count(),
        2
    );
    assert_eq!(
        shared_graph
            .matches("build_with_plan_files_prepared_config_facts_resolution_cache_and_session(")
            .count(),
        1
    );
    assert!(!shared.contains("graph_config_options"));
    assert!(!shared.contains("load_v2_config("));

    let test_filter = standalone
        .split("fn test_filters_from_prepared(")
        .nth(1)
        .expect("prepared standalone test-filter helper");
    assert!(test_filter.contains("prepared.discover_tests("));
    assert!(test_filter.contains("discover_tests_from_visible"));
    assert!(!test_filter.contains("load_v2_config"));
    assert!(!test_filter.contains("discover_visible_paths"));
}

#[test]
fn shared_traversal_reuses_runner_helpers_for_lazy_symbols_and_test_filters() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/shared-traversal-prepared-projects"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let cwd = std::env::current_dir().unwrap();
    crate::ast::begin_parse_count(&root);
    let mut framework_plan = crate::codebase::test_discovery::FrameworkPreparationPlan::default();
    framework_plan.include_framework_names(["vitest"]);
    let mut shared = SharedTraversalContext::prepare_with_framework_plan(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan {
            imports: true,
            symbols: true,
            ..Default::default()
        },
        framework_plan,
    )
    .unwrap();

    let mut lazy = traverse_args(root.clone(), vec![PathBuf::from("src/unit.test.ts")]);
    lazy.relationships = vec![RelationshipArg::Import];
    lazy.tests = vec!["vitest".to_string()];
    let lazy_result =
        collect_and_filter_entries_shared(&lazy, Direction::Deps, &cwd, &mut shared).unwrap();
    assert!(lazy_result.entries.is_empty());

    let mut symbol = traverse_args(root.clone(), vec![PathBuf::from("src/unit.ts#unit")]);
    symbol.relationships = vec![RelationshipArg::Import];
    symbol.tests = vec!["vitest".to_string()];
    let symbol_result =
        collect_and_filter_entries_shared(&symbol, Direction::Dependents, &cwd, &mut shared)
            .unwrap();
    let included_test = root.join("src/unit.test.ts");
    assert!(symbol_result.file_paths().any(|path| path == included_test));

    let mut excluded = traverse_args(
        root.clone(),
        vec![PathBuf::from("src/excluded.ts#excluded")],
    );
    excluded.relationships = vec![RelationshipArg::Import];
    excluded.tests = vec!["vitest".to_string()];
    let excluded_result =
        collect_and_filter_entries_shared(&excluded, Direction::Dependents, &cwd, &mut shared)
            .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert!(excluded_result.entries.is_empty());
    assert_eq!(shared.graph_builds, 1);
    assert_eq!(shared.symbol_index_builds, 1);
    let classifications = shared.import_resolution_cache.classification_count();
    assert!(classifications > 0);
    assert!(
        shared.import_resolution_cache.request_count() > classifications,
        "graph consumers should reuse request-scoped import classifications"
    );
    assert_eq!(counts.len(), 6, "{counts:#?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
}
#[test]
fn no_framework_import_dependents_skip_runner_configs_unless_explicit() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/no-framework-import-runner-configs"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let cwd = std::env::current_dir().unwrap();
    let mut timings = crate::codebase::timing::PhaseTimings::start();
    let mut args = traverse_args(root.clone(), vec![PathBuf::from("src/unit.ts")]);
    args.relationships = vec![RelationshipArg::Import];

    crate::ast::begin_parse_count(&root);
    let result =
        collect_and_filter_entries(&args, Direction::Dependents, &cwd, &mut timings).unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert!(result.entries.iter().any(|entry| {
        entry.node.as_file() == Some(root.join("src/consumer.ts").as_path())
    }));
    assert_eq!(counts.get(&root.join("src/unit.ts")), Some(&1), "{counts:#?}");
    assert_eq!(
        counts.get(&root.join("src/consumer.ts")),
        Some(&1),
        "{counts:#?}"
    );
    assert!(!counts.contains_key(&root.join("vitest.config.ts")), "{counts:#?}");
    assert!(
        !counts.contains_key(&root.join("playwright.config.ts")),
        "{counts:#?}"
    );

    let mut explicit = traverse_args(
        root.clone(),
        vec![
            PathBuf::from("vitest.config.ts"),
            PathBuf::from("playwright.config.ts"),
        ],
    );
    explicit.relationships = vec![RelationshipArg::Import];
    let mut timings = crate::codebase::timing::PhaseTimings::start();
    crate::ast::begin_parse_count(&root);
    collect_and_filter_entries(
        &explicit,
        Direction::Dependents,
        &cwd,
        &mut timings,
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(
        counts.get(&root.join("vitest.config.ts")),
        Some(&1),
        "{counts:#?}"
    );
    assert_eq!(
        counts.get(&root.join("playwright.config.ts")),
        Some(&1),
        "{counts:#?}"
    );
}

#[test]
fn cli_import_filter_prepares_only_the_requested_framework() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/framework-demand-invalid-unrequested"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let cwd = std::env::current_dir().unwrap();
    let mut timings = crate::codebase::timing::PhaseTimings::start();
    let mut args = traverse_args(root.clone(), vec![PathBuf::from("src/unit.ts")]);
    args.relationships = vec![RelationshipArg::Import];
    args.tests = vec!["vitest".to_string()];

    crate::ast::begin_parse_count(&root);
    crate::codebase::dotnet::test_support::begin_fact_collection_count(&root);
    crate::codebase::swift::test_support::begin_fact_collection_count(&root);
    collect_and_filter_entries(&args, Direction::Dependents, &cwd, &mut timings).unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(counts.get(&root.join("vitest.config.ts")), Some(&1));
    assert_eq!(counts.get(&root.join("playwright.config.ts")), Some(&1));
    assert_eq!(
        crate::codebase::dotnet::test_support::finish_fact_collection_count(&root),
        0
    );
    assert_eq!(
        crate::codebase::swift::test_support::finish_fact_collection_count(&root),
        0
    );
}

#[test]
fn explicit_unrequested_runner_config_is_restored_to_the_graph_scope() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/framework-demand-invalid-unrequested"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let cwd = std::env::current_dir().unwrap();
    let mut timings = crate::codebase::timing::PhaseTimings::start();
    let mut args = traverse_args(
        root.clone(),
        vec![PathBuf::from("playwright.config.ts")],
    );
    args.relationships = vec![RelationshipArg::Import];
    args.tests = vec!["vitest".to_string()];

    crate::ast::begin_parse_count(&root);
    collect_and_filter_entries(&args, Direction::Dependents, &cwd, &mut timings).unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(
        counts.get(&root.join("playwright.config.ts")),
        Some(&1),
        "{counts:#?}"
    );
}
