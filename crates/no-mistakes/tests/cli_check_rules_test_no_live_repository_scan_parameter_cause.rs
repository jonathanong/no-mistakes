use std::path::PathBuf;

const RULE: &str = "test-no-live-repository-scan";
const CASES: [&str; 17] = [
    "live-path",
    "omitted-argument",
    "unproven-variable",
    "forwarded-parameter",
    "namespace-import",
    "default-import",
    "dynamic-import",
    "require-import",
    "vi-mock",
    "non-direct-reference",
    "not-named-function",
    "reassigned-parameter",
    "multibyte-argument",
    "anonymous-callback",
    "forwarded-anonymous",
    "temp-climbs",
    "climbing-argument",
];

fn root(name: &str) -> PathBuf {
    crate::check_json_snapshot::rules_path(RULE, "parameter-cause", name)
}

#[test]
fn parameter_cause_check_json_snapshot() {
    let cases: Vec<_> = CASES
        .into_iter()
        .map(|name| {
            let root = root(name);
            (name, root.clone(), root.join(".no-mistakes.yml"))
        })
        .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_parameter_cause",
        &cases,
    );
}
