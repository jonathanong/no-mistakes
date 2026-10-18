#[test]
fn prepared_graph_config_reuses_a_supplied_test_filter() {
    let files_config = include_str!("../../codebase/dependencies/graph/files_config.rs");
    let prepared = include_str!("../../codebase/dependencies/graph/files_config_prepared.rs");
    let graph_plan = include_str!("../graph_plan.rs");
    let check_prepare = concat!(
        include_str!("../../napi_api/analyze_project/context/check_prepare.rs"),
        include_str!("../../napi_api/analyze_project/context/check_prepare_graph.rs"),
    );
    let standalone = include_str!("../../codebase/rules/run/standalone.rs");

    let fallback = files_config
        .split("fn graph_config_options_from_loaded_with_test_filter(")
        .nth(1)
        .and_then(|source| source.split("fn prefixed_queue_globs_enqueues(").next())
        .expect("graph config test-filter helper");
    assert!(
        fallback.contains("unwrap_or_else"),
        "supplied TestFileFilter must be kept"
    );
    assert_eq!(
        fallback.matches("TestFileFilter::new(").count(),
        1,
        "TestFileFilter::new is only the unprepared fallback"
    );

    assert!(!prepared.contains("TestFileFilter::new("));
    assert!(prepared.contains("graph_config_options_from_loaded_with_test_filter("));
    for source in [graph_plan, check_prepare, standalone] {
        assert!(
            source.contains("prepare_graph_config_with_test_filter("),
            "prepared check/graph path must pass a TestFileFilter"
        );
        assert!(
            !source.contains("prepare_graph_config("),
            "prepared check/graph path must not rebuild graph config without a filter"
        );
    }
}

#[test]
fn check_facts_program_delegates_ts_extract_to_the_shared_collector() {
    let program = include_str!("../../codebase/check_facts/file/program.rs");

    assert!(program.contains("facts::collect_file_facts_from_program("));
    assert!(!program.contains("extract_import_facts_from_program"));
    assert!(!program.contains("extract_symbols_from_program"));
    assert!(!program.contains("collect_call_site_facts"));
    assert!(!program.contains("collect_domain_facts"));
}

#[test]
fn prepared_check_and_analyze_graph_builders_require_complete_facts() {
    let prepared = include_str!("../../codebase/dependencies/graph/builder.rs");
    let check_facts = include_str!("../../codebase/dependencies/graph/builder_check_facts.rs");
    let standalone = prepared
        .split("pub(crate) fn build_with_plan_files_config_and_facts(")
        .nth(1)
        .and_then(|source| source.split("impl ").next())
        .expect("standalone config-and-facts builder");

    assert_eq!(
        prepared
            .matches("SuppliedFactPolicy::RequireComplete")
            .count(),
        2
    );
    assert_eq!(
        check_facts
            .matches("SuppliedFactPolicy::RequireComplete")
            .count(),
        2
    );
    assert!(standalone.contains("SuppliedFactPolicy::FillSparse"));
    assert!(!standalone.contains("SuppliedFactPolicy::RequireComplete"));
}
