use std::path::PathBuf;

#[test]
fn queue_debounce_ttl_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/debounce-ttl/fixture");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_debounce_ttl",
        &[("debounce-ttl", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let report = &reports["debounce-ttl"]["report"];
    assert_eq!(reports["debounce-ttl"]["exitCode"], 1);
    let findings = report["queues"].as_array().unwrap();
    assert!(
        findings.iter().all(|f| f["kind"] == "debounce-ttl"),
        "{findings:?}"
    );
    assert!(
        findings.iter().all(|f| f["file"] != "suppressed.ts"),
        "suppressed calls should not be reported: {findings:?}"
    );
    let warnings = report["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 3, "{warnings:?}");
    assert!(warnings.iter().all(|w| {
        let text = w.as_str().unwrap_or("");
        text.starts_with("queues-check: producers.ts:")
            && text.contains("debounce-ttl was not judged")
    }));
}
