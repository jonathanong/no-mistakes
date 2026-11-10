use std::path::PathBuf;

const RULE: &str = "test-no-live-repository-scan";
const CASES: [&str; 19] = [
    "derived-join",
    "helper-return",
    "defaulted-param",
    "join-call",
    "options-map",
    "omitted-default",
    "nested-member",
    "outside-cjs",
    "imported-cjs",
    "setup-spread",
    "sibling-const",
    "import-alias",
    "reaching-alias",
    "recursive-helper",
    "path-only-cjs",
    "path-and-unread",
    "many-blockers",
    "unmatched-config",
    "unresolved-setups",
];

fn root(name: &str) -> PathBuf {
    crate::check_json_snapshot::rules_path(RULE, "parameter-roots", name)
}

#[test]
fn parameter_roots_check_json_snapshot() {
    let cases: Vec<_> = CASES
        .into_iter()
        .map(|name| {
            let root = root(name);
            (name, root.clone(), root.join(".no-mistakes.yml"))
        })
        .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_parameter_roots",
        &cases,
    );
}
