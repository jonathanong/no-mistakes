use std::{collections::BTreeMap, path::PathBuf, process::Command};

#[test]
fn queue_lexical_and_flow_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-lexical-and-flow");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_lexical_and_flow",
        &[("default", root, PathBuf::new())],
        &[],
        &[],
    );
    let queues = reports["default"]["report"]["queues"].as_array().unwrap();
    let labels = queues
        .iter()
        .filter(|row| row["kind"] == "invalid-job-id")
        .map(|row| (row["line"].as_u64().unwrap(), row["job"].as_str()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        labels,
        BTreeMap::from([
            (11, Some("nested")),
            (12, Some("nested")),
            (13, Some("nested")),
            (14, Some("nested")),
            (15, Some("nested")),
            (19, None),
            (20, None),
            (21, None),
            (22, None),
            (23, None),
            (27, Some("forward")),
            (30, Some("outer")),
        ])
    );
    let mut missing = queues
        .iter()
        .filter(|row| row["kind"] == "missing-job-option")
        .map(|row| (row["file"].as_str().unwrap(), row["job"].as_str().unwrap()))
        .collect::<Vec<_>>();
    missing.sort_unstable();
    assert_eq!(missing, [("flow.ts", "left"), ("flow.ts", "paint-child")]);
}

#[test]
fn queue_required_options_resolved_receiver_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-required-resolution");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_required_options_resolved_receiver",
        &[("resolution", root.clone(), PathBuf::new())],
        &[],
        &[],
    );
    let report = &reports["resolution"]["report"];
    let findings = report["queues"].as_array().unwrap();
    assert_eq!(findings.len(), 3);
    for (job, file, message) in [
        (
            "real-bulk",
            "queue-primary.ts",
            "job options missing backoff",
        ),
        (
            "real-scheduler",
            "queue-secondary.ts",
            "job options missing attempts",
        ),
        (
            "real-local-scheduler",
            "queue-primary.ts",
            "job options missing backoff",
        ),
    ] {
        let finding = findings
            .iter()
            .find(|finding| finding["job"] == job)
            .unwrap();
        assert_eq!(finding["kind"], "missing-job-option");
        assert_eq!(finding["queueFile"], file);
        assert_eq!(finding["message"], message);
    }
    assert_eq!(report["warnings"], serde_json::json!([]));

    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "--lock-timeout",
            "0",
            "--jobs",
            "1",
            "queues",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "json",
            "check",
            "--include-diagnostics",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["diagnostics"],
        serde_json::json!([
            {
                "file": "enqueue.ts",
                "line": 15,
                "message": "job options were not judged because defaultJobOptions is ambiguous across multiple queue definitions",
                "severity": "warning"
            },
            {
                "file": "enqueue.ts",
                "line": 17,
                "message": "job options were not judged because defaultJobOptions is ambiguous across multiple queue definitions",
                "severity": "warning"
            }
        ])
    );
}
