use super::*;

#[test]
fn batched_deadline_views_reuse_the_prepared_owner_sources_and_programs() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/integration-tests/declared-deadlines/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let options: AnalyzeProjectOptions = serde_json::from_value(serde_json::json!({
        "root": root, "config": "single.yml", "reports": [
            {"type": "check", "includeRunnerConfigDeadlines": true},
            {"type": "check", "includeRunnerConfigDeadlines": true, "includeSuppressed": true}
        ]
    }))
    .unwrap();
    crate::ast::begin_parse_count_this_thread(&root);
    let context = AnalyzeProjectContext::prepare(&options).unwrap();
    let scope = context.scope(&options.reports[0], &options).unwrap();
    let sources = scope.traversal.visible_paths_arc().source_store_for(&root);
    let prepared_reads = sources.physical_read_count();
    assert!(prepared_reads >= 2);
    let first = context
        .project_report(&options.reports[0], &options)
        .unwrap();
    let second = context
        .project_report(&options.reports[1], &options)
        .unwrap();
    assert_eq!(
        first["runnerConfigDeadlines"],
        second["runnerConfigDeadlines"]
    );
    assert_eq!(sources.physical_read_count(), prepared_reads);
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.get(&root.join("vitest.config.ts")), Some(&1));
    assert_eq!(counts.get(&root.join("budgets.ts")), Some(&1));
}
