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
fn glob_root_errors_when_no_files_match() {
    let error = coverage_findings("glob-no-match").unwrap_err();

    assert!(
        error.to_string().contains("glob root matched no files"),
        "{error:#}"
    );
}

#[test]
fn malformed_glob_is_a_configuration_error() {
    let error = coverage_findings("glob-malformed").unwrap_err();

    assert!(
        error.to_string().contains("invalid glob") || error.to_string().contains("unclosed"),
        "{error:#}"
    );
}
