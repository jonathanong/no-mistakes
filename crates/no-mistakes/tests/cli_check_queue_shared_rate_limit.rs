use std::path::PathBuf;

#[test]
fn queue_shared_rate_limit_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/shared-rate-limit/fixture");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_shared_rate_limit",
        &[(
            "shared-rate-limit",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
        &[],
        &[],
    );
    let report = &reports["shared-rate-limit"]["report"];
    assert_eq!(reports["shared-rate-limit"]["exitCode"], 1);
    let findings = report["queues"].as_array().unwrap();
    assert!(
        findings.iter().any(|finding| {
            finding["kind"] == "shared-queue-rate-limit"
                && finding["file"] == "barrel-errors.ts"
                && finding["line"] == 32
        }),
        "barrel re-export must be reported: {findings:?}"
    );
    assert!(
        findings.iter().any(|finding| {
            finding["kind"] == "shared-queue-rate-limit"
                && finding["file"] == "barrel-renamed-only.ts"
                && finding["line"] == 5
        }),
        "renamed barrel re-export without RateLimitError text must be reported: {findings:?}"
    );
    let warnings = report["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|warning| {
            let text = warning.as_str().unwrap_or("");
            text.starts_with("queues-check: barrel-errors.ts:")
                && text.contains("could not be traced to a queue package")
        }),
        "untraceable RateLimitError must surface as an unanalyzable warning: {warnings:?}"
    );
    for key in ["rules", "codebase", "integration", "react", "advisories"] {
        assert!(
            report[key].as_array().unwrap().is_empty(),
            "{key}: {report}"
        );
    }
}
