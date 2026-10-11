use std::path::PathBuf;
use std::process::Command;

#[test]
fn cli_deadline_evidence_is_opt_in_and_preserves_ordinary_report() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/integration-tests/declared-deadlines/fixture");
    let run = |evidence: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
        command.args(["check", "--root"]).arg(&root).arg("--json");
        if evidence {
            command.arg("--include-runner-config-deadlines");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()
    };
    let baseline = run(false);
    let mut evidence = run(true);
    let runners = evidence
        .as_object_mut()
        .unwrap()
        .remove("runnerConfigDeadlines")
        .unwrap();
    assert_eq!(evidence, baseline);
    assert_eq!(runners.as_array().unwrap().len(), 2);
    assert!(runners
        .as_array()
        .unwrap()
        .iter()
        .all(|runner| runner["status"] == "prepared"));
}
