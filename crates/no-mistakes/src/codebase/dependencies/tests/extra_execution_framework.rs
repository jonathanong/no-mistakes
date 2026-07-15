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
