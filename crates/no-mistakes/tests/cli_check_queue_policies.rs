use serde_json::json;
use std::path::PathBuf;
use std::process::Command;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/queue-policy-config")
}

#[test]
fn queue_policies_selected_config_check_json_snapshot() {
    let root = fixture();
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_policies_selected_config",
        &[
            ("both", root.clone(), PathBuf::new()),
            ("advisory", root.clone(), root.join("advisory.yml")),
            ("exempt", root.clone(), root.join("exempt.yml")),
            ("off", root.clone(), root.join("off.yml")),
        ],
        &[],
        &[],
    );
    assert_eq!(reports["both"]["exitCode"], 1);
    let both = reports["both"]["report"]["queues"].as_array().unwrap();
    assert_eq!(both.len(), 2, "{both:?}");
    assert!(both
        .iter()
        .any(|finding| finding["kind"] == "unbounded-job-retention"));
    assert_eq!(reports["advisory"]["exitCode"], 0);
    let advisory = reports["advisory"]["report"]["queues"].as_array().unwrap();
    assert_eq!(advisory.len(), 1, "{advisory:?}");
    assert_eq!(advisory[0]["kind"], "queue-job-state-as-record");
    assert_eq!(advisory[0]["severity"], "advisory");
    for name in ["exempt", "off"] {
        assert_eq!(reports[name]["exitCode"], 0);
        assert_eq!(reports[name]["report"]["queues"], json!([]));
    }
}

#[test]
fn queue_policies_fixture_corpora_check_json_snapshot() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/queue-ast-hop");
    let job_state = fixtures.join("job-state-as-record/fixture");
    let retention = fixtures.join("unbounded-job-retention/fixture");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_policy_corpora",
        &[
            ("job-state", job_state, PathBuf::new()),
            ("retention", retention, PathBuf::new()),
        ],
        &[],
        &[],
    );
    let found = |report: &str, kind: &str| {
        reports[report]["report"]["queues"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|finding| finding["kind"] == kind)
            .count()
    };
    assert_eq!(found("job-state", "queue-job-state-as-record"), 6);
    assert!(found("retention", "unbounded-job-retention") > 1);
}

#[test]
fn queue_policies_reject_invalid_configuration() {
    for (name, invalid) in [
        ("invalid-retention.yml", "queues.unboundedJobRetention.keys"),
        (
            "invalid-allow-path.yml",
            "queues.jobStateAsRecord.allowPaths[0]",
        ),
    ] {
        let root = fixture();
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args([
                "--jobs",
                "1",
                "--lock-timeout",
                "0",
                "check",
                "--format",
                "json",
                "--root",
            ])
            .arg(&root)
            .arg("--config")
            .arg(root.join(name))
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains(invalid), "{stderr}");
    }
}
