use super::*;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-calls/playwright-collection/fixture"),
    )
}

fn fixture_findings(yaml: &str) -> anyhow::Result<Vec<crate::codebase::rules::RuleFinding>> {
    let root = fixture_root();
    let config = tempfile::Builder::new().suffix(".yml").tempfile()?;
    std::fs::write(config.path(), yaml)?;
    run_check(&root, Some(config.path()), None)
}

const PLAYWRIGHT_CONFIG: &str = "tests:\n  playwright:\n    configs: playwright.config.ts\n";

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
    let findings = fixture_findings(&format!(
        "{PLAYWRIGHT_CONFIG}rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{{ playwright: true }}]\n      traversal: file\n      targets:\n        - global: setTimeout\n        - exact: page.waitForTimeout\n        - moduleExport: {{ module: node:timers, export: setTimeout }}\n",
    ))
    .unwrap();

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
    let findings = fixture_findings(&format!(
        "{PLAYWRIGHT_CONFIG}rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{{ playwright: true }}]\n      traversal: file\n      unknownCalls: ignore\n      targets:\n        - global: setTimeout\n        - terminal: waitForTimeout\n        - moduleExport: {{ module: node:timers, export: setTimeout }}\n",
    ))
    .unwrap();

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
    let findings = fixture_findings(&format!(
        "{PLAYWRIGHT_CONFIG}rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{{ playwright: [chromium] }}]\n      targets: [{{ global: setTimeout }}]\n",
    ))
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "e2e/sleep.spec.ts");
}

#[test]
fn playwright_roots_still_reject_unknown_names() {
    let error = fixture_findings(&format!(
        "{PLAYWRIGHT_CONFIG}rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{{ playwright: [missing] }}]\n      targets: [{{ global: setTimeout }}]\n",
    ))
    .unwrap_err();

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
    let error = fixture_findings(
        "tests:\n  playwright:\n    configs: missing.config.ts\n    projects:\n      empty:\n        include: [does-not-exist/**/*.ts]\nrules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ playwright: true }]\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("Playwright root matched no files"),
        "{error:#}"
    );
}
