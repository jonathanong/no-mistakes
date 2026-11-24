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
            ("caller.ts", 63, Some("edge-jobs")),
            ("caller.ts", 66, Some("edge-jobs")),
            ("caller.ts", 68, Some("edge-jobs")),
            ("caller.ts", 69, Some("edge-jobs")),
            ("caller.ts", 70, Some("edge-jobs")),
            ("caller.ts", 71, Some("edge-jobs")),
            ("caller.ts", 123, None),
            ("caller.ts", 124, None),
            ("caller.ts", 125, None),
            ("caller.ts", 126, None),
            ("caller.ts", 127, None),
            ("caller.ts", 128, None),
            ("caller.ts", 129, None),
            ("caller.ts", 130, None),
            ("caller.ts", 131, None),
            ("caller.ts", 132, None),
            ("caller.ts", 133, None),
            ("caller.ts", 173, None),
            ("caller.ts", 175, None),
            ("caller.ts", 179, None),
            ("caller.ts", 181, None),
            ("caller.ts", 187, None),
            ("caller.ts", 188, None),
            ("caller.ts", 189, None),
            ("caller.ts", 190, Some("edge-jobs")),
            ("caller.ts", 192, None),
            ("caller.ts", 193, None),
            ("caller.ts", 194, None),
            ("caller.ts", 204, None),
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
            ("caller.ts", 90),
            ("caller.ts", 91),
            ("caller.ts", 92),
            ("caller.ts", 93),
            ("caller.ts", 143),
            ("caller.ts", 144),
            ("caller.ts", 145),
            ("caller.ts", 146),
            ("caller.ts", 147),
            ("caller.ts", 150),
            ("caller.ts", 153),
            ("caller.ts", 156),
            ("caller.ts", 163),
            ("caller.ts", 201),
            ("caller.ts", 209),
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
