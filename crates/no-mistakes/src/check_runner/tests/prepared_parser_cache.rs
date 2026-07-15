use super::*;
use std::path::Path;

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

#[test]
fn standalone_preparation_keeps_same_root_config_scopes_distinct() {
    let fixture = crate::test_support::materialize_gitignore_fixture("integration-aggregate");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = fixture.path().canonicalize().unwrap();
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::disabled();

    let automatic = prepared::prepare_with_session(&session, &root, None, None).unwrap();
    let explicit = prepared::prepare_with_session(
        &session,
        &root,
        Some(Path::new("explicit.no-mistakes.yml")),
        None,
    )
    .unwrap();

    assert!(automatic.config.tests.playwright.configs.is_none());
    assert!(explicit.config.tests.playwright.configs.is_some());
}

#[test]
fn standalone_preparation_keeps_same_root_tsconfig_scopes_distinct() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/symbols-output/fixture"),
    );
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::disabled();

    prepared::prepare_with_session(&session, &root, None, None).unwrap();
    let error = prepared::prepare_with_session(
        &session,
        &root,
        None,
        Some(Path::new("tsconfig-invalid.json")),
    )
    .err()
    .expect("an explicit malformed tsconfig must not reuse the automatic scope");

    assert!(format!("{error:#}").contains("tsconfig-invalid.json"));
}
