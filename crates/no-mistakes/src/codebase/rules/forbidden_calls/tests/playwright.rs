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
fn playwright_true_selects_only_playwright_tests() {
    let findings = fixture_findings("playwright-true-file-targets").unwrap();

    let mut files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    files.sort_unstable();
    files.dedup();
    assert_eq!(files, ["e2e/sleep.spec.ts"], "{findings:#?}");
    assert!(findings
        .iter()
        .any(|finding| finding.target.as_deref() == Some("global `setTimeout`")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("exact `page.waitForTimeout`")));
}

#[test]
fn playwright_terminal_matches_typed_page_parameter() {
    let findings = fixture_findings("playwright-terminal-page").unwrap();

    assert!(findings
        .iter()
        .any(|finding| finding.target.as_deref() == Some("global `setTimeout`")));
    assert!(findings.iter().any(|finding| {
        finding.target.as_deref() == Some("module export `node:timers#setTimeout`")
    }));
    assert!(
        findings.iter().any(|finding| {
            finding.target.as_deref() == Some("terminal `waitForTimeout`")
                && finding.import.as_deref() == Some("page.waitForTimeout")
        }),
        "{findings:#?}"
    );
}

#[test]
fn named_playwright_roots_use_the_prepared_catalog() {
    // This case isolates named project selection and the configured target.
    let findings = fixture_findings("playwright-named-chromium").unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "e2e/sleep.spec.ts");
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
