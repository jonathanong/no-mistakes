use super::*;

#[test]
fn validation_rejects_empty_glob_and_playwright_selectors() {
    for yaml in [
        "roots: [{ playwright: false }]\ntargets: [{ global: setTimeout }]",
        "roots: [{ glob: '' }]\ntargets: [{ global: setTimeout }]",
        "roots: [{ playwright: [] }]\ntargets: [{ global: setTimeout }]",
        "roots: [{ glob: [] }]\ntargets: [{ global: setTimeout }]",
    ] {
        assert!(config::validate(&options(yaml)).is_err(), "{yaml}");
    }
}

#[test]
fn glob_root_selects_matching_files_before_analysis() {
    let findings = coverage_findings("glob-unit-tests").unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/unit/timer.test.mts");
}

#[test]
fn glob_root_lists_union_matching_files() {
    let findings = coverage_findings("glob-unit-and-integration").unwrap();

    let mut files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    files.sort_unstable();
    assert_eq!(
        files,
        ["src/integration/timer.test.mts", "src/unit/timer.test.mts"]
    );
}

#[test]
fn glob_root_errors_when_no_files_match() {
    let error = coverage_findings("glob-no-match").unwrap_err();

    assert!(
        error.to_string().contains("glob root matched no files"),
        "{error:#}"
    );
}

#[test]
fn glob_root_keeps_exclude_as_a_finding_filter() {
    let findings = coverage_findings("glob-exclude-integration").unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/unit/timer.test.mts");
}

#[test]
fn malformed_glob_is_a_configuration_error() {
    let error = coverage_findings("glob-malformed").unwrap_err();

    assert!(
        error.to_string().contains("invalid glob") || error.to_string().contains("unclosed"),
        "{error:#}"
    );
}
