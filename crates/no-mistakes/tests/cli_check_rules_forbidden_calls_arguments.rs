use crate::git_tracked_rule;
use serde_json::Value;
use std::process::Command;

#[test]
fn argument_predicates_report_live_calls_and_keep_fixture_launches_allowed() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "argument-predicates");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(root.path())
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{:?}", output);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 2, "{report:#?}");
    assert_eq!(
        findings
            .iter()
            .map(|f| f["line"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![14, 15]
    );
    for finding in findings {
        assert_eq!(finding["file"], "repro.mjs");
        assert_eq!(
            finding["target"],
            "module export `node:child_process#execFile`"
        );
        assert!(finding["message"]
            .as_str()
            .unwrap()
            .contains("no-live-analyzer-from-tests"));
    }
    let allowed = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(root.path())
        .arg("--config")
        .arg(root.path().join("escape-ignore.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(allowed.status.success(), "{:?}", allowed);
    let report: Value = serde_json::from_slice(&allowed.stdout).unwrap();
    assert_eq!(report["rules"], serde_json::json!([]));
}
