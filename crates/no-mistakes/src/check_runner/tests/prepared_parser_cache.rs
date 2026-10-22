use super::*;
use std::path::{Path, PathBuf};

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
    let prepared =
        prepared::prepare_with_session(&session, &root, None, None, None, false).unwrap();
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

    let automatic =
        prepared::prepare_with_session(&session, &root, None, None, None, false).unwrap();
    let explicit = prepared::prepare_with_session(
        &session,
        &root,
        Some(Path::new("explicit.no-mistakes.yml")),
        None,
        None,
        false,
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

    prepared::prepare_with_session(&session, &root, None, None, None, false).unwrap();
    let error = prepared::prepare_with_session(
        &session,
        &root,
        None,
        Some(Path::new("tsconfig-invalid.json")),
        None,
        false,
    )
    .err()
    .expect("an explicit malformed tsconfig must not reuse the automatic scope");

    assert!(format!("{error:#}").contains("tsconfig-invalid.json"));
}

#[test]
fn preparation_builds_at_most_one_workspace_projection() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/unique-exports-basic/fixture"),
    );
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::new(Some(
        std::sync::Arc::clone(&observer),
    ));

    prepared::prepare_with_session(&session, &root, None, None, None, false).unwrap();

    assert_eq!(observer.snapshot().work["workspace.builds"], 1);
}

#[test]
fn explicit_tsconfig_without_playwright_skips_workspace_projection() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/queries/fixture"),
    );
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::new(Some(
        std::sync::Arc::clone(&observer),
    ));

    prepared::prepare_with_session(
        &session,
        &root,
        None,
        Some(Path::new("tsconfig.json")),
        None,
        false,
    )
    .unwrap();

    assert_eq!(observer.snapshot().work.get("workspace.builds"), None);
}

fn case_root(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases")
            .join(rel),
    )
}

fn git_sha_fail_root() -> PathBuf {
    case_root("rules/no-test-git-sha/fixture/fail")
}

fn workspace_builds(root: &Path, deadlines: bool) -> Option<u64> {
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::new(Some(
        std::sync::Arc::clone(&observer),
    ));
    prepared::prepare_with_session(&session, root, None, None, None, deadlines).unwrap();
    observer.snapshot().work.get("workspace.builds").copied()
}

#[test]
fn filesystem_only_config_skips_workspace_projection() {
    assert_eq!(workspace_builds(&git_sha_fail_root(), false), None);
}

#[test]
fn filesystem_only_config_skips_workspace_when_deadlines_have_no_runner_configs() {
    assert_eq!(workspace_builds(&git_sha_fail_root(), true), None);
}

#[test]
fn declared_runner_configs_build_workspace_only_for_deadline_evidence() {
    let root = case_root("integration-tests/declared-deadlines/fixture");
    assert_eq!(workspace_builds(&root, false), None);
    assert_eq!(workspace_builds(&root, true), Some(1));
}

#[test]
fn prepared_embedded_sql_rule_builds_workspace_for_catalog() {
    let root = case_root("rules/postgres-idempotent-insert/fixture/fail");
    assert_eq!(workspace_builds(&root, false), Some(1));
}

#[test]
fn filesystem_only_check_matches_standalone_filesystem_findings() {
    let root = git_sha_fail_root();
    let config_path = root.join(".no-mistakes.yml");
    let aggregate = run_all(root.clone(), Some(config_path.clone()), None).unwrap();
    let standalone =
        no_mistakes::codebase::rules::run_filesystem_rules(&root, Some(&config_path)).unwrap();
    assert_eq!(
        aggregate.rules, standalone,
        "{:?}\n{standalone:?}",
        aggregate.rules
    );
}
