use std::path::PathBuf;

#[test]
fn queue_wait_forms_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/wait-forms/fixture");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_wait_forms",
        &[
            ("configured", root.clone(), root.join(".no-mistakes.yml")),
            ("unconfigured", root.clone(), root.join("unconfigured.yml")),
        ],
        &[],
        &[],
    );
    for (case, count) in [("configured", 15), ("unconfigured", 14)] {
        assert_eq!(reports[case]["exitCode"], 1);
        let report = &reports[case]["report"];
        let findings = report["queues"].as_array().unwrap();
        assert_eq!(findings.len(), count, "{report}");
        assert!(report["warnings"].as_array().unwrap().is_empty());
        assert!(findings.iter().all(|finding| {
            finding["file"] != "api/routes/negative.ts"
                && finding["file"] != "api/worker.ts"
                && finding["file"] != "api/routes/suppressed.ts"
        }));
        assert_eq!(
            findings
                .iter()
                .any(|finding| finding["file"] == "api/routes/wrapper.ts"),
            case == "configured"
        );
    }
}
