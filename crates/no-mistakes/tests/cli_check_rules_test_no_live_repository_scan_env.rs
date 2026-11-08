use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;
use std::process::Command;

const RULE: &str = "test-no-live-repository-scan";
const CASES: &[(&str, usize, usize)] = &[
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
            let parameter = message.find("as a parameter").unwrap();
            let stub = message.find("via vi.stubEnv").unwrap();
            let suppression = message.find("no-mistakes-disable-next-line").unwrap();
            assert!(parameter < stub && stub < suppression, "{message}");
        }
    }
}
