use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;
use std::process::Command;

const RULE: &str = "test-no-live-repository-scan";
const CASES: &[(&str, usize)] = &[
    ("module-deleted", 1),
    ("module-missing", 1),
    ("function-owned", 0),
    ("function-deleted", 1),
    ("function-missing", 1),
    ("function-live-write", 1),
    ("function-missing-or", 1),
    ("function-computed-fallback", 1),
    ("function-cleanup-safe", 0),
    ("function-mixed-test-callers", 1),
    ("captured-program-clear-reset", 1),
    ("function-first-statement", 1),
];

fn root(name: &str) -> PathBuf {
    rules_path(RULE, "env-cause", name)
}

fn cases(config: &str) -> Vec<(&'static str, PathBuf, PathBuf)> {
    CASES
        .iter()
        .map(|(name, _)| {
            let root = root(name);
            (*name, root.clone(), root.join(config))
        })
        .collect()
}

fn report(name: &str, config: &str) -> (i32, serde_json::Value) {
    let root = root(name);
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(config))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{name}: {error}: {output:?}"));
    (output.status.code().unwrap(), report)
}

#[test]
fn env_cause_check_json_snapshot() {
    assert_saved_fixture(
        "test_no_live_repository_scan_env_cause",
        &cases(".no-mistakes.yml"),
    );
}

#[test]
fn env_cause_rule_off_check_json_snapshot() {
    assert_saved_fixture(
        "test_no_live_repository_scan_env_cause_off",
        &cases(".off.yml"),
    );
}

#[test]
fn env_cause_snapshot_keeps_the_relative_tmp_fallback_literal() {
    let (_, report) = report("module-deleted", ".no-mistakes.yml");
    let normalized =
        super::check_json_snapshot::normalize_report(&serde_json::to_string(&report).unwrap(), &[]);
    let report: serde_json::Value = serde_json::from_str(&normalized).unwrap();
    let message = report["advisories"][0]["message"].as_str().unwrap();
    assert!(
        message.contains("literal fallback \"./tmp/example\""),
        "{message}"
    );
}

#[test]
fn env_cause_preserves_findings_and_names_the_test_and_fallback() {
    for (name, expected) in CASES {
        let (exit, report) = report(name, ".no-mistakes.yml");
        assert_eq!(exit, 0, "{name}: {report}");
        assert!(
            report["rules"].as_array().unwrap().is_empty(),
            "{name}: {report}"
        );
        let findings = report["advisories"].as_array().unwrap();
        assert_eq!(findings.len(), *expected, "{name}: {report}");
        for finding in findings {
            assert_eq!(finding["file"], "scan.mts", "{name}: {finding}");
            assert_eq!(finding["target"], "unresolved root");
            let message = finding["message"].as_str().unwrap();
            assert!(
                message.contains("process.env.EXAMPLE_LOCAL_DIR"),
                "{name}: {message}"
            );
            assert!(message.contains("root.mts:"), "{name}: {message}");
            if *name == "function-computed-fallback" {
                assert!(!message.contains("literal fallback"), "{message}");
            } else if matches!(
                *name,
                "function-deleted" | "function-missing" | "function-live-write"
            ) {
                assert!(
                    message.contains("literal fallback \"\""),
                    "{name}: {message}"
                );
            } else {
                assert!(message.contains("./tmp/example"), "{name}: {message}");
            }
            let fix = message.split_once("Fix:").unwrap().1;
            assert!(fix.contains("root.mts:"), "{name}: {message}");
            assert!(fix.contains("vi.stubEnv"), "{name}: {message}");
            assert!(fix.contains("EXAMPLE_LOCAL_DIR"), "{name}: {message}");
            assert!(
                fix.contains("no-mistakes-disable-next-line"),
                "{name}: {message}"
            );
        }
    }
}

#[test]
fn env_cause_points_to_the_clearing_statement() {
    for (name, location) in [
        ("function-deleted", "root.mts:12"),
        ("module-deleted", "root.mts:4"),
        ("function-mixed-test-callers", "root.mts:10"),
    ] {
        let (_, report) = report(name, ".no-mistakes.yml");
        let message = report["advisories"][0]["message"].as_str().unwrap();
        assert!(message.contains(location), "{name}: {message}");
        assert!(
            message.contains("clears that environment variable"),
            "{name}: {message}"
        );
    }
}

#[test]
fn env_cause_points_to_the_unassigned_test_entry() {
    for (name, location) in [
        ("function-missing", "root.mts:4"),
        ("module-missing", "root.mts:4"),
        ("function-first-statement", "root.mts:1"),
    ] {
        let (_, report) = report(name, ".no-mistakes.yml");
        let message = report["advisories"][0]["message"].as_str().unwrap();
        assert!(message.contains(location), "{name}: {message}");
    }
}

#[test]
fn env_cause_does_not_blame_a_clear_superseded_by_owned_setup() {
    let (_, report) = report("captured-program-clear-reset", ".no-mistakes.yml");
    let findings = report["advisories"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{report}");
    let message = findings[0]["message"].as_str().unwrap();
    assert!(
        !message.contains("clears that environment variable"),
        "{message}"
    );
    assert!(message.contains("root.mts:9"), "{message}");
    assert!(message.contains("captured"), "{message}");
}

#[test]
fn env_cause_policy_and_custom_message_check_json_snapshot() {
    let root = root("function-missing");
    assert_saved_fixture(
        "test_no_live_repository_scan_env_cause_policy",
        &[
            ("report", root.clone(), root.join(".report.yml")),
            ("custom-message", root.clone(), root.join(".strict.yml")),
        ],
    );
}

#[test]
fn env_cause_respects_report_policy_and_custom_messages() {
    let (exit, strict) = report("function-missing", ".report.yml");
    assert_eq!(exit, 1, "{strict}");
    assert!(strict["advisories"].as_array().unwrap().is_empty());
    assert_eq!(strict["rules"].as_array().unwrap().len(), 1);
    assert!(strict["rules"][0]["message"]
        .as_str()
        .unwrap()
        .contains("process.env.EXAMPLE_LOCAL_DIR"));
    let (exit, custom) = report("function-missing", ".strict.yml");
    assert_eq!(exit, 1, "{custom}");
    assert!(custom["advisories"].as_array().unwrap().is_empty());
    assert_eq!(custom["rules"].as_array().unwrap().len(), 1);
    assert_eq!(
        custom["rules"][0]["message"],
        "Custom environment root finding."
    );
}

#[test]
fn env_cause_ambiguous_capture_keeps_generic_diagnostic_check_json_snapshot() {
    let root = root("captured-ambiguous");
    assert_saved_fixture(
        "test_no_live_repository_scan_env_cause_ambiguous",
        &[(
            "captured-ambiguous",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
    );
    let (exit, report) = report("captured-ambiguous", ".no-mistakes.yml");
    assert_eq!(exit, 0, "{report}");
    let findings = report["advisories"].as_array().unwrap();
    assert_eq!(findings.len(), 4, "{report}");
    for finding in findings {
        let message = finding["message"].as_str().unwrap();
        assert!(
            message.contains("expression this analysis cannot follow"),
            "{message}"
        );
        assert!(!message.contains("process.env.FIRST"), "{message}");
        assert!(!message.contains("process.env.SECOND"), "{message}");
    }
}
