use super::*;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-calls/playwright-collection/fixture"),
    )
}

fn fixture_findings(case: &str) -> anyhow::Result<Vec<crate::codebase::rules::RuleFinding>> {
    run_check(
        &fixture_root(),
        Some(&super::forbidden_calls_case(case)),
        None,
    )
}

#[test]
fn playwright_roots_require_a_catalog() {
    let (root, graph) = call_graph();
    let files = vec![root.join("src/entry.mts")];
    let playwright = options("roots: [{ playwright: true }]\ntargets: [{ global: setTimeout }]");
    let error = expand_roots(&root, &playwright, &graph, &files).unwrap_err();
    assert!(error
        .to_string()
        .contains("require a prepared Playwright project catalog"));
}

#[test]
fn playwright_roots_still_reject_unknown_names() {
    let error = fixture_findings("playwright-unknown-project").unwrap_err();

    assert!(
        error.to_string().contains("names an unknown project"),
        "{error:#}"
    );
}

#[test]
fn playwright_root_errors_when_no_files_match() {
    // Point `configs` at a missing file so runner discovery is skipped, then use
    // an explicit include that matches nothing. A missing config alone is an I/O
    // error, not the empty-collection diagnostic.
    let error = fixture_findings("playwright-no-files").unwrap_err();

    assert!(
        error
            .to_string()
            .contains("Playwright root matched no files"),
        "{error:#}"
    );
}
