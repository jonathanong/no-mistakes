use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

const RULE: &str = "test-no-live-repository-scan";
const CASES: &[&str] = &[
    "destructured",
    "forwarding",
    "cli",
    "cli-guarded",
    "explicit-roots",
    "explicit-destructured",
    "explicit-cli",
    "global-setup-escape",
    "explicit-roots-imported",
    "explicit-roots-live",
    "explicit-roots-unknown",
    "explicit-roots-loader",
    "production-default",
    "production-owned",
    "production-module",
    "live-test",
    "unknown-test",
    "cli-reached",
];

fn root(name: &str) -> PathBuf {
    crate::check_json_snapshot::rules_path(RULE, "test-origin-callers", name)
}

#[test]
fn test_no_live_repository_scan_test_origin_check_json_snapshot() {
    let cases: Vec<_> = CASES
        .iter()
        .map(|name| {
            let root = root(name);
            (*name, root.clone(), root.join(".no-mistakes.yml"))
        })
        .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_test_origins",
        &cases,
    );
}

#[test]
fn test_no_live_repository_scan_only_test_originating_callers_decide_ownership() {
    for name in CASES {
        let root = root(name);
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--format", "json", "--root"])
            .arg(&root)
            .output()
            .unwrap();
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        let findings: Vec<_> = ["rules", "advisories"]
            .into_iter()
            .flat_map(|field| report[field].as_array().unwrap())
            .filter(|item| item["rule"] == RULE)
            .collect();
        let expected = usize::from(matches!(
            *name,
            "global-setup-escape"
                | "explicit-roots-imported"
                | "explicit-roots-live"
                | "explicit-roots-unknown"
                | "explicit-roots-loader"
                | "production-default"
                | "production-module"
                | "live-test"
                | "unknown-test"
                | "cli-reached"
        ));
        assert_eq!(findings.len(), expected, "{name}: {findings:?}");
    }
}
