use std::path::PathBuf;

#[test]
fn queue_required_options_selected_config_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-required-config");
    let on = root.join("default-on");
    let off = root.join("default-off");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_required_options_selected_config",
        &[
            ("default-on", on.clone(), PathBuf::new()),
            ("explicit-off", on.clone(), on.join("explicit.yml")),
            ("default-off", off.clone(), PathBuf::new()),
            ("explicit-on", off.clone(), off.join("explicit.yml")),
        ],
        &[],
        &[],
    );
    for name in ["default-on", "explicit-on"] {
        assert_eq!(reports[name]["exitCode"], 1);
        let queue = reports[name]["report"]["queues"].as_array().unwrap();
        assert_eq!(queue.len(), 1, "{name}: {queue:?}");
        assert_eq!(queue[0]["kind"], "missing-job-option");
        assert_eq!(queue[0]["message"], "job options missing attempts");
    }
    for name in ["explicit-off", "default-off"] {
        assert_eq!(reports[name]["exitCode"], 0);
        assert_eq!(reports[name]["report"]["queues"], serde_json::json!([]));
    }
}

#[test]
fn queue_unbounded_retention_selected_config_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-retention-config");
    let on = root.join("default-on");
    let off = root.join("default-off");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_unbounded_retention_selected_config",
        &[
            ("default-on", on.clone(), PathBuf::new()),
            ("explicit-off", on.clone(), on.join("explicit.yml")),
            ("default-off", off.clone(), PathBuf::new()),
            ("explicit-on", off.clone(), off.join("explicit.yml")),
        ],
        &[],
        &[],
    );
    for name in ["default-on", "explicit-on"] {
        assert_eq!(reports[name]["exitCode"], 1);
        let queue = reports[name]["report"]["queues"].as_array().unwrap();
        assert_eq!(queue.len(), 1, "{name}: {queue:?}");
        assert_eq!(queue[0]["kind"], "unbounded-job-retention");
    }
    for name in ["explicit-off", "default-off"] {
        assert_eq!(reports[name]["exitCode"], 0);
        assert_eq!(reports[name]["report"]["queues"], serde_json::json!([]));
    }
}

#[test]
fn queue_job_state_as_record_selected_config_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-job-state-config");
    let on = root.join("default-on");
    let off = root.join("default-off");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_job_state_as_record_selected_config",
        &[
            ("default-on", on.clone(), PathBuf::new()),
            ("explicit-off", on.clone(), on.join("explicit.yml")),
            ("default-off", off.clone(), PathBuf::new()),
            ("explicit-on", off.clone(), off.join("explicit.yml")),
        ],
        &[],
        &[],
    );
    for name in ["default-on", "explicit-on"] {
        assert_eq!(reports[name]["exitCode"], 0, "advisory check must not fail");
        assert_eq!(reports[name]["report"]["queues"], serde_json::json!([]));
        let advisories = reports[name]["report"]["advisories"].as_array().unwrap();
        assert_eq!(advisories.len(), 1, "{name}: {advisories:?}");
        assert_eq!(advisories[0]["rule"], "queues-check");
    }
    for name in ["explicit-off", "default-off"] {
        assert_eq!(reports[name]["exitCode"], 0);
        assert_eq!(reports[name]["report"]["queues"], serde_json::json!([]));
        assert_eq!(reports[name]["report"]["advisories"], serde_json::json!([]));
    }
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "queues",
            "--root",
            on.to_str().unwrap(),
            "--format",
            "json",
            "check",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "standalone queues check must exit 0 for advisory-only findings: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let findings: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = findings.as_array().unwrap();
    assert_eq!(arr.len(), 1, "{findings}");
    assert_eq!(arr[0]["kind"], "job-state-as-record");
    assert_eq!(arr[0]["severity"], "advisory");
}
