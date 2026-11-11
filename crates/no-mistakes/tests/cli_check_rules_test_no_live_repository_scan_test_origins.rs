use std::path::PathBuf;

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
    "cli-argv-cleanup",
    "cli-argv-explicit-function",
    "cli-argv-explicit-file",
    "cli-argv-scalar-command",
    "cli-argv-block-destructure",
    "cli-argv-reused-parameter",
    "cli-argv-captured-property",
    "cli-argv-object-rest",
    "cli-argv-pattern-parameter",
    "cli-argv-literal-initializer",
    "cli-argv-arrow-cleanup",
    "cli-argv-numeric-mismatch",
    "cli-argv-numeric-live",
    "cli-argv-empty-command",
    "cli-argv-unknown-command",
    "cli-argv-local-source",
    "cli-argv-single-default",
    "cli-argv-preceding-statement",
    "cli-argv-nested-pattern",
    "cli-argv-initializer-factory",
    "cli-argv-member-object",
    "cli-argv-deep-and",
    "cli-argv-nonfinite-arity",
    "cli-argv-default-parameter",
    "cli-argv-nested-owner",
    "cli-argv-deep-owner",
    "cli-argv-spread",
    "cli-argv-mutated-alias",
    "cli-argv-default-mutation",
    "cli-argv-escaped",
    "cli-argv-reassigned",
    "cli-argv-literal-live",
    "cli-argv-live",
    "cli-argv-long",
    "cli-argv-mixed",
    "cli-argv-mutated",
    "cli-argv-short",
    "cli-argv-unknown",
    "cli-argv-unknown-array",
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
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "test_no_live_repository_scan_test_origins",
        &cases,
        &[],
        &[],
    );
    for name in CASES {
        let report = &reports[*name]["report"];
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
        let expected = if name.starts_with("cli-argv-")
            && !matches!(
                *name,
                "cli-argv-cleanup"
                    | "cli-argv-short"
                    | "cli-argv-long"
                    | "cli-argv-arrow-cleanup"
                    | "cli-argv-numeric-mismatch"
                    | "cli-argv-empty-command"
            ) {
            2
        } else {
            expected
        };
        assert_eq!(findings.len(), expected, "{name}: {findings:?}");
    }
}
