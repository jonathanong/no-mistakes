use super::*;

#[test]
fn graph_check_partial_property_matches_standalone_without_aborting_traversal_preparation() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/partial-config-failure"),
    );
    let config = root.join("graph-scopes-check.yml");
    let analyzed = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "config": config,
            "reports": [{"type": "check"}]
        })
        .to_string(),
    ))
    .unwrap();
    let analyzed: Value = serde_json::from_str(&analyzed).unwrap();
    let checked = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "config": config
        })
        .to_string(),
    ))
    .unwrap();
    let checked: Value = serde_json::from_str(&checked).unwrap();
    let actual = &analyzed["reports"][0]["result"];
    assert_eq!(actual, &checked);
    assert!(
        actual["incomplete"]
            .as_array()
            .unwrap()
            .iter()
            .any(|failure| {
                failure["property"] == "testDir"
                    && failure["file"]
                        .as_str()
                        .is_some_and(|file| file.ends_with("playwright.projects.config.ts"))
            }),
        "{actual}"
    );
    assert!(
        actual["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["rule"] == "no-empty-or-comments-only-files"),
        "{actual}"
    );
    let graph_findings = actual["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "forbidden-dependencies")
        .collect::<Vec<_>>();
    assert!(
        !graph_findings.is_empty(),
        "healthy graph application must retain its finding: {actual}"
    );
    assert!(
        graph_findings.iter().all(|finding| finding["file"]
            .as_str()
            .is_some_and(|file| file.ends_with("good/graph.ts"))),
        "failed graph application must not emit findings: {actual}"
    );
}

#[test]
fn mixed_failed_playwright_selection_has_no_coverage_findings_but_healthy_sibling_runs() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/mixed-selection-prerequisites"),
    );
    let output = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root, "config": ".no-mistakes.yml" }).to_string(),
    ))
    .unwrap();
    let output: Value = serde_json::from_str(&output).unwrap();
    assert!(
        output["incomplete"]
            .as_array()
            .unwrap()
            .iter()
            .any(|failure| failure["consumer"] == "playwright-coverage"
                && failure["property"] == "testDir"),
        "{output}"
    );
    let coverage = output["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "playwright-coverage")
        .collect::<Vec<_>>();
    assert!(
        !coverage.is_empty(),
        "healthy explicitly selected project must retain its finding: {output}"
    );
    assert!(
        coverage.iter().all(|finding| finding["file"]
            .as_str()
            .is_some_and(|file| file.starts_with("healthy/"))),
        "failed mixed selection must not produce uncovered findings: {output}"
    );
}
