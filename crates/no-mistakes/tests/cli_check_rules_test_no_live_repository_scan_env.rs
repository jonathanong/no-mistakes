use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;
use std::process::Command;

const RULE: &str = "test-no-live-repository-scan";
const CASES: &[(&str, usize, usize)] = &[
    ("sync-hook-reset-intersection", 0, 0),
    ("sync-hook-helper-member-suffix", 0, 1),
    ("sync-hook-helper-unknown-suffix", 0, 1),
    ("sync-hook-parameter-cleanup-read", 0, 1),
    ("sync-hook-configured-only-cleanup", 0, 1),
    ("sync-hook-configured-parameter-cleanup", 0, 1),
    ("sync-hook-config-concurrent-cleanup", 0, 1),
    ("sync-hook-config-unknown-concurrent-cleanup", 0, 1),
    ("sync-hook-config-sequential-cleanup", 0, 0),
    ("sync-hook-config-default-sequential-cleanup", 0, 0),
    ("sync-hook-config-inline-project-concurrent-cleanup", 0, 1),
    ("sync-hook-config-imported-project-concurrent-cleanup", 0, 1),
    ("sync-hook-config-extends-concurrent-cleanup", 0, 1),
    ("sync-hook-concurrent-tagged-each-cleanup", 0, 1),
    ("sync-hook-concurrent-sibling-suite-cleanup", 0, 1),
    ("sync-hook-concurrent-named-callback-cleanup", 0, 1),
    ("sync-hook-concurrent-options-named-callback-cleanup", 0, 1),
    ("sync-hook-concurrent-shared-setup-cleanup", 0, 1),
    ("sync-hook-sequential-unrelated-concurrent", 0, 0),
    ("sync-hook-sequential-unrelated-concurrent-suite", 0, 0),
    ("sync-hook-concurrent-cleanup", 0, 1),
    ("sync-hook-concurrent-suite-cleanup", 0, 1),
    ("sync-hook-concurrent-options-cleanup", 0, 1),
    ("sync-hook-unknown-concurrent-options-cleanup", 0, 1),
    ("sync-hook-aliased-concurrent-cleanup", 0, 1),
    ("captured-wrapper-dynamic-parent", 0, 1),
    ("captured-wrapper-dynamic-child", 0, 0),
    ("captured-wrapper-normalized-parent", 0, 0),
    ("sync-local-helper-cleanup-normalized-parent", 0, 0),
    ("sync-local-helper-cleanup-live-suffix", 0, 1),
    ("captured-wrapper-parent", 0, 1),
    ("captured-wrapper-absolute-root", 1, 0),
    ("captured-wrapper-typed-owned", 0, 0),
    ("captured-wrapper-effect-call", 0, 2),
    ("captured-wrapper-opaque-constructor", 0, 1),
    ("sync-imported-helper-alias-suffix", 0, 0),
    ("sync-imported-helper-forwarded-suffix", 0, 0),
    ("sync-imported-helper-unknown-suffix", 0, 1),
    ("sync-imported-helper-live-suffix", 0, 1),
    ("sync-imported-helper-absolute-suffix", 0, 1),
    ("sync-hook-fake-local-cleanup", 0, 1),
    ("sync-hook-shadowed-cleanup", 0, 1),
    ("sync-hook-custom-import-cleanup", 0, 1),
    ("sync-hook-aliased-cleanup", 0, 0),
    ("sync-imported-helper-absolute-resolve", 0, 1),
    ("sync-hook-beforeall-aftereach-delete", 0, 1),
    ("sync-hook-beforeall-aftereach-unstub", 0, 1),
    ("sync-hook-beforeeach-cleanup-live-write", 0, 1),
    ("sync-hook-beforeeach-cleanup-conditional", 0, 1),
    ("sync-hook-beforeeach-cleanup-arbitrary-reset", 0, 1),
    ("sync-hook-beforeeach-cleanup-read", 0, 1),
    ("sync-hook-beforeall-cleanup-read", 0, 1),
    ("captured-wrapper-aliased-write", 0, 1),
    ("captured-wrapper-escaped", 0, 1),
    ("captured-wrapper-unknown-write", 0, 1),
    ("captured-wrapper-promises-owned", 0, 0),
    ("captured-wrapper-namespace-owned", 0, 0),
    ("captured-wrapper-owned", 0, 0),
    ("captured-wrapper-inspected", 0, 0),
    ("captured-wrapper-live", 1, 0),
    ("captured-wrapper-mutated", 0, 1),
    ("captured-wrapper-replaced", 0, 1),
    ("sync-imported-helper-live", 0, 1),
    ("sync-imported-helper-missing", 0, 1),
    ("sync-imported-helper-late", 0, 1),
    ("sync-imported-helper-mixed", 0, 1),
    ("sync-imported-helper-parent", 0, 1),
    ("sync-imported-helper-captured", 0, 1),
    ("sync-imported-helper-impure", 0, 1),
    ("sync-hook-beforeeach-helper-local-cleanup", 0, 0),
    ("sync-hook-beforeeach-helper-imported-cleanup", 0, 0),
    ("sync-hook-beforeeach-tmpdir-direct-cleanup", 0, 0),
    ("sync-hook-beforeeach-tmpdir-child-cleanup", 0, 0),
    ("sync-hook-beforeall-helper-local-cleanup", 0, 0),
    ("sync-hook-beforeall-helper-imported-cleanup", 0, 0),
    ("sync-hook-beforeall-tmpdir-direct-cleanup", 0, 0),
    ("sync-hook-beforeall-tmpdir-child-cleanup", 0, 0),
    ("sync-hook-beforeeach-helper-local", 0, 0),
    ("sync-hook-beforeeach-helper-imported", 0, 0),
    ("sync-hook-beforeeach-tmpdir-direct", 0, 0),
    ("sync-hook-beforeeach-tmpdir-child", 0, 0),
    ("sync-hook-beforeall-helper-local", 0, 0),
    ("sync-hook-beforeall-helper-imported", 0, 0),
    ("sync-hook-beforeall-tmpdir-direct", 0, 0),
    ("sync-hook-beforeall-tmpdir-child", 0, 0),
    ("helper-fallback-assignment", 0, 0),
    ("helper-fallback-table", 0, 0),
    ("helper-fallback-assignment-live", 0, 1),
    ("helper-fallback-assignment-missing", 0, 1),
    ("helper-fallback-assignment-late", 0, 1),
    ("helper-fallback-assignment-mixed", 0, 1),
    ("owned-tmpdir", 0, 0),
    ("owned-tmpdir-parent", 1, 0),
    ("owned-tmpdir-nested-parent", 1, 0),
    ("owned-tmpdir-child-parent", 0, 0),
    ("owned-env-tmpdir-parent", 0, 1),
    ("owned-tmpdir-child-parent-escape", 1, 0),
    ("owned-tmpdir-escaped-env-setup", 1, 0),
    ("owned-tmpdir-nested-receiver", 1, 0),
    ("owned-tmpdir-array-binding", 1, 0),
    ("owned-tmpdir-member-loader", 1, 0),
    ("owned-tmpdir-default", 0, 0),
    ("owned-tmpdir-custom-import", 1, 0),
    ("owned-tmpdir-shadowed-require", 1, 0),
    ("owned-tmpdir-local-fake", 1, 0),
    ("owned-tmpdir-namespace-fake", 1, 0),
    ("owned-tmpdir-shadowed", 1, 0),
    ("owned-tmpdir-require", 0, 0),
    ("owned-tmpdir-require-named", 0, 0),
    ("owned-tmpdir-renamed", 0, 0),
    ("live-root-with-env-tail", 1, 0),
    ("owned-tmpdir-named", 0, 0),
    ("owned-tmpdir-captured", 0, 1),
    ("owned-tmpdir-stub-live", 1, 0),
    ("owned-tmpdir-late", 1, 0),
    ("owned-tmpdir-mixed", 1, 0),
    ("owned-tmpdir-live", 1, 0),
    ("owned-tmpdir-missing", 1, 0),
    ("owned-env-tmpdir", 0, 0),
    ("owned-env-tmpdir-stub-live", 0, 1),
    ("owned-env-tmpdir-late", 0, 1),
    ("owned-env-tmpdir-mixed", 0, 1),
    ("owned-env-tmpdir-live", 0, 1),
    ("owned-env-tmpdir-missing", 0, 1),
    ("nested-env-mkdtemp", 0, 0),
    ("same-test-fake-stub", 0, 1),
    ("same-test-shadowed-vi", 0, 1),
    ("same-test-getter-reset", 0, 1),
    ("same-test-configured-reset", 0, 0),
    ("same-test-aliased-vi", 0, 0),
    ("same-test-owned", 0, 0),
    ("same-test-repository", 0, 1),
    ("same-test-conditional", 0, 1),
    ("same-test-restored", 0, 0),
    ("same-test-async", 0, 0),
    ("same-test-unrelated-write", 0, 0),
    ("same-test-reset-between", 0, 1),
    ("same-test-helper-reset", 0, 1),
    ("same-test-default-reset", 0, 1),
    ("same-test-argument-reset", 0, 1),
    ("same-test-await-between", 0, 1),
    ("same-test-unknown-vi", 0, 1),
    ("stub-direct", 0, 0),
    ("assignment-direct", 0, 0),
    ("stub-hook", 0, 0),
    ("stub-setup", 0, 0),
    ("assignment-setup", 0, 0),
    ("transitive-setup", 0, 0),
    ("configured-stub", 0, 0),
    ("configured-assignment", 0, 0),
    ("configured-project-ownership", 0, 0),
    ("parameter", 0, 0),
    ("live-root", 1, 0),
    ("workspace-static", 0, 0),
    ("workspace-dynamic", 0, 1),
    ("late-write", 0, 1),
    ("setup-before-write", 0, 1),
    ("configured-conditional", 0, 1),
    ("configured-repository", 0, 1),
    ("configured-cycle", 0, 1),
    ("configured-global-only", 0, 1),
    ("unstubbed", 0, 1),
    ("repository", 0, 1),
    ("conditional-setup", 0, 1),
    ("uncalled-setup", 0, 1),
    ("type-only-setup", 0, 1),
    ("dynamic-setup", 0, 1),
    ("cyclic-setup", 0, 1),
    ("mocked-setup", 0, 1),
    ("typed-mocked-setup", 0, 1),
];

fn cases(config: &str) -> Vec<(&'static str, PathBuf, PathBuf)> {
    CASES
        .iter()
        .map(|(name, _, _)| {
            let root = rules_path(RULE, "env-provenance", name);
            (*name, root.clone(), root.join(config))
        })
        .collect()
}

#[test]
fn env_provenance_check_json_snapshot() {
    assert_saved_fixture(
        "test_no_live_repository_scan_env",
        &cases(".no-mistakes.yml"),
    );
}

#[test]
fn env_provenance_rule_off_check_json_snapshot() {
    assert_saved_fixture("test_no_live_repository_scan_env_off", &cases(".off.yml"));
}

#[test]
fn env_provenance_preserves_unknown_roots_and_actionable_hints() {
    for (name, violations, expected) in CASES {
        let root = rules_path(RULE, "env-provenance", name);
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .args(["--format", "json"])
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(i32::from(*violations > 0)),
            "{name}: {output:?}"
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            *expected,
            "{name}: {report}"
        );
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            *violations,
            "{name}: {report}"
        );
        for advisory in report["advisories"].as_array().unwrap() {
            let message = advisory["message"].as_str().unwrap();
            if matches!(
                *name,
                "sync-hook-parameter-cleanup-read" | "sync-hook-configured-parameter-cleanup"
            ) {
                assert!(message.contains("passes `process.env.X`, an unproven variable"));
                assert!(message.contains("pass a const fixture path or an owned mkdtemp root"));
                continue;
            }
            let local = message.find("inside the enumerating function").unwrap();
            let callers = message
                .find("every statically resolved test caller")
                .unwrap();
            let stub = message.find("vi.stubEnv").unwrap();
            let suppression = message.find("no-mistakes-disable-next-line").unwrap();
            assert!(
                local < callers && callers < stub && stub < suppression,
                "{message}"
            );
        }
    }
}
