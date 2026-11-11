use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;
use std::process::Command;

const RULE: &str = "test-no-live-repository-scan";
const CASES: &[(&str, usize, usize)] = &[
    ("helper-fallback-assignment", 0, 0),
    ("helper-fallback-table", 0, 0),
    ("helper-fallback-assignment-live", 0, 1),
    ("helper-fallback-assignment-missing", 0, 1),
    ("helper-fallback-assignment-late", 0, 1),
    ("helper-fallback-assignment-mixed", 0, 1),
    ("owned-tmpdir", 0, 0),
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
