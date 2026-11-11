use std::path::PathBuf;

const CASES: &[&str] = &[
    "optional-prototype-live",
    "object-accessor-mutation",
    "object-prototype-default",
    "captured-mixed-env",
    "optional-missing-owned",
    "rest-object-mutation",
    "rest-object-escape",
    "rest-object-spread-escape",
    "mock-local-alias-owned",
    "mock-local-alias-live",
    "array-object-owned",
    "array-object-live",
    "unknown-position-object",
    "rest-object-owned",
    "rest-object-live",
    "default-property-unknown",
    "mock-rewrite-owned",
    "mock-rewrite-live",
    "default-omitted-owned",
    "default-omitted-live",
    "default-omitted-unknown",
    "default-scalar-owned",
    "default-scalar-live",
    "fallback-two-params-owned",
    "fallback-two-params-live",
    "fallback-two-params-unknown",
    "fallback-absolute-tail",
    "fallback-relative-tail",
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

#[test]
fn forwarded_roots_preserve_positive_and_negative_controls() {
    let allowed = [
        "array-object-owned",
        "array-spread-owned",
        "default-omitted-owned",
        "default-owned",
        "default-scalar-owned",
        "fallback-or-owned",
        "fallback-owned",
        "fallback-relative-tail",
        "fallback-two-params-owned",
        "mock-local-alias-owned",
        "mock-rewrite-owned",
        "object-empty-owned",
        "object-literal-owned",
        "object-spread-owned",
        "optional-missing-owned",
        "rest-object-owned",
        "rest-owned",
        "rewrite-owned",
        "rewrite-owned-branch",
    ];
    for &name in CASES {
        let root = crate::check_json_snapshot::rules_path(
            "test-no-live-repository-scan",
            "forwarded-roots",
            name,
        );
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(".no-mistakes.yml"))
            .args(["--format", "json", "--jobs", "1"])
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(
            report["warnings"].as_array().unwrap().is_empty(),
            "{name}: {report}"
        );
        let found = ["rules", "advisories"].iter().any(|key| {
            report[*key]
                .as_array()
                .unwrap()
                .iter()
                .any(|finding| finding["rule"] == "test-no-live-repository-scan")
        });
        assert_eq!(found, !allowed.contains(&name), "{name}: {report}");
    }
}
