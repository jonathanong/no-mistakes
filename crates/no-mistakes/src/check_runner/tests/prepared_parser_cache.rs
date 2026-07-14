use super::*;

#[test]
fn standalone_preparation_parses_each_config_once_across_distinct_selections() {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/playwright-multi-selection"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();

    no_mistakes::ast::begin_parse_count(&root);
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::disabled();
    let prepared = prepared::prepare_with_session(&session, &root, None, None).unwrap();
    let counts = no_mistakes::ast::finish_parse_count(&root);
    let expected = [
        root.join("playwright.admin.config.ts"),
        root.join("playwright.web.config.ts"),
    ];

    assert!(prepared.playwright.is_some());
    assert_eq!(counts.len(), expected.len(), "{counts:#?}");
    for path in expected {
        assert_eq!(counts.get(&path), Some(&1), "{counts:#?}");
    }
}
