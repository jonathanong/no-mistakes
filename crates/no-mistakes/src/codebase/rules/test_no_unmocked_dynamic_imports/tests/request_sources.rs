use super::*;

#[test]
fn standalone_request_reads_and_parses_runner_and_graph_files_once() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-project-setups");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let config = crate::config::v2::load_v2_config(
        &root,
        Some(&root.join(".no-mistakes-repository-playwright.yml")),
    )
    .unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let findings = {
        let _guard = crate::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        crate::ast::begin_parse_count(&root);
        let findings = check(&root, &config, None).unwrap();
        let counts = crate::ast::finish_parse_count(&root);
        for file in [
            "repository-vitest.config.mts",
            "playwright.target.config.mts",
            "test-helpers/web.setup.mts",
            "test-helpers/web-mocks.mts",
            "web/covered.test.mts",
        ] {
            assert_eq!(counts.get(&root.join(file)), Some(&1), "{counts:#?}");
        }
        assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
        findings
    };
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        vec!["web/covered.test.mts"]
    );
    assert_eq!(observer.snapshot().work["analysis.requests"], 1);
    assert_eq!(observer.snapshot().work["graph.builds"], 1);
    assert!(
        observer
            .source_read_snapshot()
            .values()
            .all(|reads| *reads == 1),
        "{:?}",
        observer.source_read_snapshot()
    );
}
