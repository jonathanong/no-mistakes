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

#[test]
fn queue_test_scope_and_same_file_workers_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/test-scope-and-same-file-workers/fixture");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_test_scope_and_same_file_workers",
        &[("default", root.clone(), PathBuf::new())],
        &[],
        &[],
    );
    assert_eq!(reports["default"]["exitCode"], 1);
    let queues = reports["default"]["report"]["queues"].as_array().unwrap();
    assert_eq!(queues.len(), 1, "{queues:?}");
    assert_eq!(queues[0]["kind"], "unmatched-producer");
    assert_eq!(queues[0]["file"], "apps/api/orphan-enqueue.mts");
    assert_eq!(queues[0]["job"], "never-handled");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "queues",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "json",
            "check",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let findings: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = findings.as_array().unwrap();
    assert_eq!(arr.len(), 1, "{findings}");
    assert_eq!(arr[0]["file"], "apps/api/orphan-enqueue.mts");

    let edges_output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "queues",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "json",
            "edges",
        ])
        .output()
        .unwrap();
    assert!(edges_output.status.success());
    let edges: serde_json::Value = serde_json::from_slice(&edges_output.stdout).unwrap();
    for edge in edges.as_array().unwrap() {
        assert!(!edge["from"].as_str().unwrap().contains("/integration/"));
        assert!(!edge["to"].as_str().unwrap().contains("/integration/"));
    }
}

fn run_queues_check(root: &std::path::Path, jobs: &str, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .env_remove("RAYON_NUM_THREADS")
        .args([
            "--lock-timeout",
            "0",
            "--jobs",
            jobs,
            "queues",
            "check",
            "--root",
        ])
        .arg(root)
        .args(["--format", "json"])
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn queue_required_options_standalone_config_check_json_snapshot() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let fixtures = repo.join("fixtures/check/queue-required-config");
    let mut cases = serde_json::Map::new();
    for (name, directory, args, expected) in [
        ("default-off", "default-off", &[][..], 0),
        ("default-on", "default-on", &[][..], 1),
        (
            "explicit-off",
            "default-on",
            &["--config", "explicit.yml"][..],
            0,
        ),
        (
            "explicit-on",
            "default-off",
            &["--config", "explicit.yml"][..],
            1,
        ),
    ] {
        let root = fixtures.join(directory);
        let run = |jobs| {
            let output = run_queues_check(&root, jobs, args);
            assert_eq!(
                output.status.code(),
                Some(expected),
                "{name}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let findings: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(findings.len(), expected as usize, "{name}");
            let mut value = serde_json::json!({
                "exitCode": expected,
                "report": findings,
            });
            crate::check_json_snapshot::source_lines::add_source_lines(&mut value, &root, repo);
            value.sort_all_objects();
            value
        };
        let single = run("1");
        assert_eq!(single, run("2"), "{name}");
        cases.insert(name.to_string(), single);
    }
    insta::with_settings!({
        snapshot_path => "common/snapshots",
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!(
            "queue_required_options_standalone_config",
            format!("{}\n", serde_json::to_string_pretty(&cases).unwrap())
        );
    });
}

#[test]
fn queue_required_options_standalone_rejects_a_missing_explicit_config() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-required-config/default-off");
    let output = run_queues_check(&root, "1", &["--config", "unavailable.yml"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unavailable.yml"), "{stderr}");
}
#[test]
fn queue_retention_and_listing_followups_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/queue-followups/fixture");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_retention_and_listing_followups",
        &[("queue-followups", root, PathBuf::new())],
        &[],
        &[],
    );
    let result = &reports["queue-followups"];
    assert_eq!(result["exitCode"], 1);
    let report = &result["report"];
    let findings = report["queues"].as_array().unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings
        .iter()
        .all(|finding| finding["kind"] == "unbounded-job-retention"));
    assert!(findings.iter().any(|finding| {
        finding["job"] == "known-missing"
            && finding["message"] == "unbounded job retention: removeOnFail unset"
    }));
    assert!(findings.iter().any(|finding| {
        finding["job"] == "welcome"
            && finding["queueFile"] == "queue:opaque"
            && finding["message"] == "unbounded job retention: removeOnFail kept without a bound"
    }));

    let advisories = report["advisories"].as_array().unwrap();
    let lines: Vec<_> = advisories
        .iter()
        .map(|row| row["line"].as_u64().unwrap())
        .collect();
    assert_eq!(lines, [23, 29, 35, 52, 66, 72, 76]);
    assert!(advisories.iter().all(|row| row["file"] == "listing.ts"));

    let warnings = report["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 5, "{warnings:?}");
    for (location, rule) in [
        ("enqueue.ts:8", "unbounded-job-retention"),
        ("enqueue.ts:13", "unbounded-job-retention"),
        ("listing.ts:45", "job-state-as-record"),
        ("listing.ts:59", "job-state-as-record"),
        ("listing.ts:91", "job-state-as-record"),
    ] {
        assert!(
            warnings.iter().any(|warning| {
                let warning = warning.as_str().unwrap();
                warning.contains(location) && warning.contains(&format!("{rule} was not judged"))
            }),
            "{location}: {warnings:?}"
        );
    }
}
