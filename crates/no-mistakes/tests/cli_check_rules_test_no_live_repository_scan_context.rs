use std::path::PathBuf;

const CASES: &[(&str, bool)] = &[
    ("nested-forwarded-url-leaf-escape-live", false),
    ("forwarded-url-no-escape-owned", true),
    ("forwarded-string-mutator-owned", true),
    ("local-url-leaf-escape-live", false),
    ("local-string-leaf-escape-owned", true),
    ("local-url-no-escape-owned", true),
    ("local-copied-url-owned", true),
    ("converted-url-scalar-owned", true),
    ("constructed-url-param-escape-live", false),
    ("buffer-encoding-leaf-escape-live", false),
    ("selected-scalar-url-fallback-owned", true),
    ("selected-url-fallback-escape-live", false),
    ("local-rest-exclusion-owned", true),
    ("resolve-bases-relative-owned", true),
    ("resolve-bases-absolute-live", false),
    ("coalesce-null-dot-owned", true),
    ("null-container-dot-owned", true),
    ("captured-default-owner-live", false),
    ("forwarded-default-other-parameter-live", false),
    ("local-rest-excluded-read-live", false),
    ("local-rest-source-write-live", false),
    ("local-rest-literal-owned", true),
    ("nested-literal-key-default-live", false),
    ("nested-forwarded-write-live", false),
    ("nested-forwarded-call-live", false),
    ("nested-forwarded-alias-live", false),
    ("nested-forwarded-method-live", false),
    ("nested-forwarded-whole-method-live", false),
    ("nested-forwarded-computed-live", false),
    ("nested-forwarded-pristine-owned", true),
    ("nested-forwarded-unrelated-owned", true),
    ("nested-forwarded-scalar-read-owned", true),
    ("nested-forwarded-root-alias-owned", true),
    ("nested-forwarded-root-alias-write-live", false),
    ("nested-forwarded-new-live", false),
    ("nested-forwarded-wrapped-method-live", false),
    ("nested-deep-call-escape-live", false),
    ("nested-nullish-escape-live", false),
    ("nested-method-escape-live", false),
    ("nested-write-live", false),
    ("nested-computed-write-live", false),
    ("nested-compound-write-live", false),
    ("nested-update-unknown", false),
    ("nested-delete-unknown", false),
    ("nested-destructured-write-live", false),
    ("nested-for-of-write-live", false),
    ("nested-for-in-write-live", false),
    ("nested-call-escape-live", false),
    ("nested-assign-escape-live", false),
    ("nested-new-escape-live", false),
    ("nested-alias-escape-live", false),
    ("nested-read-owned", true),
    ("nested-scalar-alias-owned", true),
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
