use std::path::PathBuf;

#[test]
fn queue_worker_rate_limit_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/worker-rate-limit/fixture");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_worker_rate_limit",
        &[(
            "worker-rate-limit",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
        &[],
        &[],
    );
    let report = &reports["worker-rate-limit"]["report"];
    assert_eq!(reports["worker-rate-limit"]["exitCode"], 1);
    let findings = report["queues"].as_array().unwrap();
    assert!(findings
        .iter()
        .all(|finding| finding["kind"] == "worker-rate-limit-per-process"));
    assert!(findings.iter().all(|finding| {
        let file = finding["file"].as_str().unwrap_or("");
        file != "queue-level-limiter.ts"
            && file != "non-worker.ts"
            && file != "unresolved.ts"
            && file != "suppressed.ts"
    }));
    assert!(report["warnings"].as_array().unwrap().is_empty());
    for key in ["rules", "codebase", "integration", "react", "advisories"] {
        assert!(
            report[key].as_array().unwrap().is_empty(),
            "{key}: {report}"
        );
    }
}
