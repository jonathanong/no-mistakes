use std::path::PathBuf;

const CASES: &[(&str, bool)] = &[
    ("schema-rest-param-owned", true),
    ("schema-rest-local-owned", true),
    ("schema-rest-live", false),
    ("schema-rest-unknown", false),
    ("schema-rest-mutated-live", false),
    ("schema-rest-default-live", false),
    ("glob-nested-null-owned", true),
    ("glob-undefined-owned", true),
    ("glob-nested-dot-owned", true),
    ("glob-owned-env-suffix-live", false),
    ("glob-dotted-key-live", false),
    ("schema-mixed-live", false),
    ("schema-undefined-live", false),
    ("glob-nested-owned", true),
    ("glob-nested-climb-live", false),
    ("glob-nested-absolute-live", false),
    ("glob-nested-unknown-live", false),
    ("glob-null-owned", true),
    ("rest-trailing-owned", true),
    ("rest-leading-owned", true),
    ("disjoint-trailing-owned", true),
    ("override-trailing-live", false),
    ("opaque-trailing-unknown", false),
    ("rest-live-caller", false),
    ("schema-direct-owned", true),
    ("schema-entry-owned", true),
    ("schema-both-owned", true),
    ("schema-missing-live", false),
    ("schema-null-live", false),
    ("schema-unknown-live", false),
    ("schema-explicit-live", false),
    ("schema-or-empty-live", false),
    ("glob-relative-owned", true),
    ("glob-dot-owned", true),
    ("glob-climb-live", false),
    ("glob-absolute-live", false),
    ("glob-unknown-live", false),
];

#[test]
fn caller_context_check_json_snapshot() {
    let cases: Vec<(&str, PathBuf, PathBuf)> = CASES
        .iter()
        .map(|&(name, _)| {
            let root = crate::check_json_snapshot::rules_path(
                "test-no-live-repository-scan",
                "caller-context",
                name,
            );
            (name, root.clone(), root.join(".no-mistakes.yml"))
        })
        .collect();
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "test_no_live_repository_scan_caller_context",
        &cases,
        &[],
        &[],
    );
    for &(name, owned) in CASES {
        let report = &reports[name]["report"];
        assert!(
            report["warnings"].as_array().unwrap().is_empty(),
            "{name}: {report}"
        );
        let found = ["rules", "advisories"].iter().any(|category| {
            report[*category]
                .as_array()
                .unwrap()
                .iter()
                .any(|finding| finding["rule"] == "test-no-live-repository-scan")
        });
        assert_eq!(found, !owned, "{name}: {report}");
    }
}
