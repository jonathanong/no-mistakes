use std::path::PathBuf;

const CASES: &[&str] = &[
    "array-spread-unknown",
    "fallback-unknown",
    "rewrite-after",
    "rewrite-overwritten",
    "rewrite-owned-branch",
    "object-nested-unknown",
    "object-empty-owned",
    "default-unknown",
    "array-spread-owned",
    "array-spread-live",
    "object-spread-owned",
    "object-spread-live",
    "object-literal-owned",
    "object-literal-live",
    "rest-owned",
    "rest-live",
    "rewrite-owned",
    "rewrite-live",
    "default-owned",
    "default-live",
    "fallback-owned",
    "fallback-live",
    "fallback-or-owned",
    "fallback-or-live",
];

#[test]
fn forwarded_roots_check_json_snapshot() {
    let cases: Vec<(&str, PathBuf, PathBuf)> = CASES
        .iter()
        .map(|&name| {
            let root = crate::check_json_snapshot::rules_path(
                "test-no-live-repository-scan",
                "forwarded-roots",
                name,
            );
            (name, root.clone(), root.join(".no-mistakes.yml"))
        })
        .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_forwarded_roots",
        &cases,
    );
}
