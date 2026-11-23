use std::collections::BTreeSet;
use std::path::PathBuf;

#[test]
fn configured_enqueue_wrappers_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/configured-enqueue-wrappers/fixture");
    let controls =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/queue-wrapper-unit-3904");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "configured_enqueue_wrappers",
        &[
            ("configured", root.clone(), root.join(".no-mistakes.yml")),
            ("before", root.clone(), root.join("../configs/before.yml")),
            (
                "controls",
                controls.clone(),
                controls.join("configs/check.yml"),
            ),
        ],
        &[],
        &[],
    );
    let control_findings = reports["controls"]["report"]["queues"].as_array().unwrap();
    assert!(control_findings.len() >= 2, "{control_findings:?}");
    assert!(control_findings
        .iter()
        .any(|f| f["kind"] == "self-dedup-enqueue"
            && f["file"] == "feedback.ts"
            && f["line"] == 9));
    let before = reports["before"]["report"]["queues"].as_array().unwrap();
    assert_eq!(before.len(), 6);
    assert!(before.iter().all(|f| f["kind"] == "unmatched-worker"));
    let report = &reports["configured"]["report"];
    assert_eq!(reports["configured"]["exitCode"], 1);
    let findings = report["queues"].as_array().unwrap();
    let kinds: BTreeSet<_> = findings
        .iter()
        .map(|f| f["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        BTreeSet::from([
            "invalid-job-id",
            "debounce-ttl",
            "missing-job-option",
            "mixed-priority",
        ])
    );
    for file in [
        "enqueues/object.ts",
        "enqueues/returned.ts",
        "enqueues/fixed.ts",
    ] {
        assert!(
            findings
                .iter()
                .any(|f| f["file"] == file && f["kind"] == "invalid-job-id"),
            "{file}: {findings:?}"
        );
        assert!(
            findings
                .iter()
                .any(|f| f["file"] == file && f["kind"] == "debounce-ttl"),
            "{file}: {findings:?}"
        );
    }
    for file in [
        "enqueues/abstain.ts",
        "enqueues/deep-alias.ts",
        "enqueues/deep-write.ts",
    ] {
        assert!(
            findings.iter().all(|f| f["file"] != file),
            "{file}: {findings:?}"
        );
    }
    let missing: BTreeSet<_> = findings
        .iter()
        .filter(|f| f["kind"] == "missing-job-option")
        .map(|f| f["job"].as_str().unwrap())
        .collect();
    assert_eq!(missing, BTreeSet::from(["returnedTask", "noDefaultsTask"]));
    for field in ["rules", "codebase", "integration", "react", "advisories"] {
        assert_eq!(report[field], serde_json::json!([]), "{field}: {report}");
    }
}

#[test]
fn configured_enqueue_wrappers_edges_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/configured-enqueue-wrappers/fixture")
        .canonicalize()
        .unwrap();
    let mut reports = Vec::new();
    for jobs in ["1", "2"] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args([
                "queues",
                "--root",
                root.to_str().unwrap(),
                "--format",
                "json",
                "edges",
            ])
            .env("RAYON_NUM_THREADS", jobs)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        reports.push(serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap());
    }
    assert_eq!(reports[0], reports[1]);
    let edges = reports[0].as_array().unwrap();
    assert_eq!(edges.len(), 12, "{edges:?}");
    for edge in edges.iter().filter(|e| e["kind"] == "queue-enqueue") {
        assert!(edges.iter().any(|worker| worker["kind"] == "queue-worker"
            && worker["from"] == edge["to"]
            && worker["to"] == "workers/worker.ts"));
    }
    assert!(edges.iter().all(|e| e["from"] != "enqueues/abstain.ts"));
    insta::with_settings!({snapshot_path => "common/snapshots", prepend_module_to_snapshot => false}, {
        insta::assert_snapshot!("configured_enqueue_wrapper_edges", serde_json::to_string_pretty(&reports[0]).unwrap());
    });
}

#[test]
fn partial_wrapper_names_and_absent_options_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/wrapper-partials/fixture");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "partial_enqueue_wrappers",
        &[("partial", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let findings = reports["partial"]["report"]["queues"].as_array().unwrap();
    assert_eq!(findings.len(), 3, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .filter(|f| f["kind"] == "mixed-priority")
            .count(),
        2
    );
    assert!(findings
        .iter()
        .any(|f| f["kind"] == "unmatched-worker-unresolved-producers" && f["job"] == "pending"));
    assert!(findings
        .iter()
        .all(|f| f["kind"] != "unmatched-worker" && f["kind"] != "missing-job-option"));
}
