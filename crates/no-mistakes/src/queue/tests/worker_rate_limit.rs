use super::*;
use serde_json::{json, Value};
use std::fs;

#[test]
fn worker_rate_limit_reports_resolved_workers_and_abstains_when_unresolved() {
    let root = fixture("worker-rate-limit");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(
        report.check.iter().any(|finding| {
            finding.kind == "worker-rate-limit-per-process"
                && finding.file.ends_with("suppressed.ts")
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
        .filter(|finding| finding.kind == "worker-rate-limit-per-process")
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
        .filter(|diagnostic| diagnostic.message.contains("worker-rate-limit-per-process"))
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
            !file.ends_with("queue-level-limiter.ts")
                && !file.ends_with("non-worker.ts")
                && !file.ends_with("unresolved.ts")
                && !file.ends_with("suppressed.ts")
        }),
        "only resolved worker receivers are findings: {check:?}"
    );
    assert!(
        diagnostics.iter().all(|diagnostic| {
            let file = diagnostic["file"].as_str().unwrap_or("");
            file.ends_with("unresolved.ts")
        }),
        "only unresolved receivers abstain: {diagnostics:?}"
    );
    assert_eq!(diagnostics.len(), 4, "diagnostics: {diagnostics:?}");
    assert!(
        check.iter().any(|finding| {
            finding["file"].as_str() == Some("subclass-worker.ts")
                && finding["sourceLine"].as_str()
                    == Some("await new ExampleWorker().rateLimit(1000);")
        }),
        "Worker subclasses must be reported: {check:?}"
    );
    insta::assert_snapshot!(serde_json::to_string_pretty(&json!({
        "check": check,
        "diagnostics": diagnostics,
    }))
    .unwrap());
}

#[test]
fn worker_rate_limit_handles_reexports_defaults_aliases_and_unresolved_edges() {
    let root = fixture("worker-rate-limit-edges");
    let report = analyze_project(&root, None, &[]).unwrap();
    let findings: Vec<_> = report
        .check
        .iter()
        .filter(|finding| finding.kind == "worker-rate-limit-per-process")
        .map(|finding| {
            (
                finding.file.as_str(),
                finding.line,
                finding.queue_name.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        findings,
        vec![
            ("caller.ts", 51, Some("edge-jobs")),
            ("caller.ts", 54, Some("edge-jobs")),
            ("caller.ts", 56, Some("edge-jobs")),
            ("caller.ts", 57, Some("edge-jobs")),
            ("caller.ts", 58, Some("edge-jobs")),
            ("caller.ts", 59, Some("edge-jobs")),
            ("caller.ts", 111, None),
            ("caller.ts", 112, None),
            ("caller.ts", 113, None),
            ("caller.ts", 114, None),
            ("caller.ts", 115, None),
            ("caller.ts", 116, None),
            ("caller.ts", 117, None),
            ("caller.ts", 118, None),
            ("caller.ts", 119, None),
            ("caller.ts", 120, None),
            ("caller.ts", 121, None),
        ],
        "only resolved re-exported worker aliases, subclasses, and awaited factories are reported: {:?}",
        report.check
    );
    let diag_lines: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.contains("worker-rate-limit-per-process"))
        .map(|diagnostic| (diagnostic.file.as_str(), diagnostic.line))
        .collect();
    assert_eq!(
        diag_lines,
        vec![
            ("caller.ts", 78),
            ("caller.ts", 79),
            ("caller.ts", 80),
            ("caller.ts", 81),
            ("caller.ts", 131),
            ("caller.ts", 132),
            ("caller.ts", 133),
            ("caller.ts", 134),
            ("caller.ts", 135),
            ("caller.ts", 138),
            ("caller.ts", 141),
            ("caller.ts", 144),
        ],
        "only unresolved queue-module bindings abstain while unrelated receivers stay silent: {:?}",
        report.diagnostics
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
