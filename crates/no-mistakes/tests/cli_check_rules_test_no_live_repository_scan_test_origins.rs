use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

const RULE: &str = "test-no-live-repository-scan";
const CASES: &[&str] = &[
    "colocated-const-owned",
    "colocated-const-live",
    "colocated-const-unknown",
    "colocated-hook-owned",
    "colocated-hook-live",
    "colocated-hook-unknown",
    "colocated-options-owned",
    "colocated-options-live",
    "colocated-options-unknown",
    "captured-env-setup-owned",
    "module-env-unproven",
    "captured-env-function-owned",
    "captured-env-setup-live",
    "captured-env-setup-unknown",
    "captured-env-hook-owned",
    "captured-env-hook-live",
    "captured-env-hook-unknown",
    "colocated-options-mutated",
    "colocated-options-escaped",
    "colocated-options-shared",
    "captured-env-setup-imported-before-write",
    "colocated-options-shared-clean",
    "colocated-options-captured-env",
    "colocated-options-env-hook",
    "captured-env-program-owned",
    "captured-env-program-live",
    "captured-env-program-unknown",
    "captured-env-import-owned",
    "captured-env-import-live",
    "captured-env-import-unknown",
    "captured-env-program-late",
    "captured-env-import-cycle",
    "captured-env-member-owned",
    "captured-env-member-live",
    "captured-env-member-unknown",
    "captured-env-member-alias-owned",
    "captured-env-member-alias-live",
    "captured-env-member-alias-unknown",
    "captured-env-setup-loaded-first",
    "destructured",
    "forwarding",
    "cli",
    "cli-guarded",
    "explicit-roots",
    "explicit-roots-barrel",
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
            "colocated-const-live"
                | "colocated-const-unknown"
                | "colocated-hook-live"
                | "colocated-hook-unknown"
                | "colocated-options-live"
                | "colocated-options-unknown"
                | "module-env-unproven"
                | "captured-env-function-owned"
                | "captured-env-setup-live"
                | "captured-env-setup-unknown"
                | "captured-env-hook-owned"
                | "captured-env-hook-live"
                | "captured-env-hook-unknown"
                | "colocated-options-mutated"
                | "colocated-options-escaped"
                | "colocated-options-shared"
                | "captured-env-setup-imported-before-write"
                | "colocated-options-shared-clean"
                | "colocated-options-captured-env"
                | "colocated-options-env-hook"
                | "captured-env-program-live"
                | "captured-env-program-unknown"
                | "captured-env-import-live"
                | "captured-env-import-unknown"
                | "captured-env-program-late"
                | "captured-env-import-cycle"
                | "captured-env-member-owned"
                | "captured-env-member-live"
                | "captured-env-member-unknown"
                | "captured-env-member-alias-owned"
                | "captured-env-member-alias-live"
                | "captured-env-member-alias-unknown"
                | "captured-env-setup-loaded-first"
                | "global-setup-escape"
                | "explicit-roots-barrel"
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
