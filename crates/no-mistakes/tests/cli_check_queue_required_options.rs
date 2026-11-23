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
