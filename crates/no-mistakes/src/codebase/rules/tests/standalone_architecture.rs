#[test]
fn standalone_rules_prepare_one_request_without_nested_rule_discovery() {
    let source = include_str!("../run/standalone.rs");
    assert_eq!(source.matches("VisiblePathSnapshot::new(root)").count(), 1);
    assert_eq!(source.matches("load_v2_config_from_visible(").count(), 1);
    assert_eq!(source.matches("InferredRoots::from_visible(").count(), 1);
    assert_eq!(source.matches("resolve_tsconfig_from_visible(").count(), 1);
    assert_eq!(source.matches("prepare_from_snapshot(").count(), 1);
    assert_eq!(
        source
            .matches("prepare_graph_config_with_test_filter(")
            .count(),
        1
    );
    assert_eq!(source.matches("standalone_fact_plan(&config)").count(), 1);
    assert_eq!(
        source
            .matches("collect_check_facts_with_graph_files_playwright_and_sources(")
            .count(),
        1
    );
    assert_eq!(source.matches("run_with_declared_plan(").count(), 1);
    for nested in [
        "test_no_unmocked_dynamic_imports::check(",
        "server_route_client_boundary::check(",
        "nextjs_no_api_routes::check(",
        "nextjs_no_caching::check(",
        "require_storybook_stories::check(",
        "playwright::rules::check(",
        "forbidden_dependencies::check_with_config(",
    ] {
        assert!(
            !source.contains(nested),
            "standalone fanout called {nested}"
        );
    }
}
