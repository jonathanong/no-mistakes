use super::*;
use serde_json::{json, Value};
use std::fs;

#[test]
fn shared_queue_rate_limit_reports_only_mixed_queues() {
    let root = fixture("shared-rate-limit");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(
        report.check.iter().any(|finding| {
            finding.kind == "shared-queue-rate-limit" && finding.file.ends_with("suppressed.ts")
        }),
        "the raw report keeps the suppressed site, check: {:?}",
        report.check
    );
    let mut findings = report.check;
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    crate::codebase::rules::suppress_domain_findings_with_sources(
        &root,
        &mut findings,
        &sources,
        |finding| crate::codebase::rules::SuppressionTarget {
            domain: "queues",
            rule: "queues-check",
            file: &finding.file,
            line: Some(finding.line),
            reason: &finding.message,
            identity: None,
        },
    );
    let check: Vec<_> = findings
        .iter()
        .filter(|finding| finding.kind == "shared-queue-rate-limit")
        .map(|finding| {
            with_source(
                &root,
                &finding.file,
                finding.line,
                serde_json::to_value(finding).unwrap(),
            )
        })
        .collect();
    let diagnostics: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.contains("rate limit"))
        .map(|diagnostic| {
            with_source(
                &root,
                &diagnostic.file,
                diagnostic.line,
                serde_json::to_value(diagnostic).unwrap(),
            )
        })
        .collect();
    assert!(
        check.iter().all(|finding| {
            let file = finding["file"].as_str().unwrap_or("");
            !file.ends_with("single.ts")
                && !file.ends_with("unknown.ts")
                && !file.ends_with("unresolved.ts")
                && !file.ends_with("suppressed.ts")
        }),
        "only mixed queues are findings: {check:?}"
    );
    insta::assert_snapshot!(serde_json::to_string_pretty(&json!({
        "check": check,
        "diagnostics": diagnostics,
    }))
    .unwrap());
}

#[test]
fn shared_queue_and_per_process_rate_limit_both_report_on_mixed_queue() {
    let root = fixture("shared-rate-limit");
    let report = analyze_project(&root, None, &[]).unwrap();
    let kinds: Vec<_> = report
        .check
        .iter()
        .filter(|finding| finding.file == "mixed-call.ts" && finding.line == 4)
        .map(|finding| finding.kind.as_str())
        .collect();
    assert_eq!(
        kinds,
        vec!["shared-queue-rate-limit", "worker-rate-limit-per-process"],
        "mixed-queue worker.rateLimit() reports both independent findings: {:?}",
        report.check
    );
}

fn with_source(root: &std::path::Path, file: &str, line: usize, mut value: Value) -> Value {
    let source = fs::read_to_string(root.join(file)).unwrap_or_default();
    let text = source
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim();
    value["sourceLine"] = Value::String(text.to_string());
    value
}
