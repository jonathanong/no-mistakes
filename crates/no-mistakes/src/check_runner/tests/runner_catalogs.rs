use crate::check_runner::prepared;
use std::path::{Path, PathBuf};

fn prepare(root: &Path) -> prepared::PreparedCheckInputs {
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::disabled();
    prepared::prepare_with_session(
        &session,
        root,
        Some(&root.join(".no-mistakes.yml")),
        None,
        None,
    )
    .unwrap()
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(name)
        .join("fixture")
}

#[test]
fn forbidden_calls_file_roots_skip_runner_catalogs() {
    let prepared = prepare(&fixture("forbidden-calls/coverage"));
    assert!(prepared.vitest_projects.is_none());
    assert!(prepared.playwright_projects.is_none());
}

#[test]
fn forbidden_calls_vitest_roots_prepare_only_the_vitest_catalog() {
    let prepared = prepare(&fixture("forbidden-calls/explicit-vitest-project"));
    assert!(prepared.vitest_projects.is_some());
    assert!(prepared.playwright_projects.is_none());
}

#[test]
fn forbidden_calls_playwright_roots_prepare_only_the_playwright_catalog() {
    let prepared = prepare(&fixture("forbidden-calls/explicit-playwright-project"));
    assert!(prepared.vitest_projects.is_none());
    assert!(prepared.playwright_projects.is_some());
}

#[test]
fn vitest_ci_coverage_still_prepares_the_vitest_catalog() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/vitest-ci-path-coverage/pass");
    let prepared = prepare(&root);
    assert!(prepared.vitest_projects.is_some());
    assert!(prepared.playwright_projects.is_none());
}
