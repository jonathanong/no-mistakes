use super::{analyze_project, fixture};
use std::path::Path;

#[test]
fn unresolved_bulk_scheduler_and_ambiguous_multi_definitions_abstain() {
    let root = fixture("job-options-followups");
    let report = analyze_project(&root, None, &[]).unwrap();
    let findings: Vec<_> = report
        .check
        .iter()
        .filter(|finding| finding.kind == "missing-job-option")
        .map(|finding| {
            serde_json::json!({
                "kind": finding.kind,
                "file": finding.file,
                "line": finding.line,
                "queueFile": finding.queue_file,
                "queueName": finding.queue_name,
                "job": finding.job,
                "message": finding.message,
                "sourceLine": source_line(&root, &finding.file, finding.line),
            })
        })
        .collect();
    let diagnostics: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.starts_with("job options"))
        .map(|diagnostic| {
            serde_json::json!({
                "file": diagnostic.file,
                "line": diagnostic.line,
                "message": diagnostic.message,
                "sourceLine": source_line(&root, &diagnostic.file, diagnostic.line),
            })
        })
        .collect();
    let snapshot = serde_json::to_string_pretty(&serde_json::json!({
        "findings": findings,
        "diagnostics": diagnostics,
    }))
    .unwrap();
    insta::assert_snapshot!(snapshot);
}

fn source_line(root: &Path, file: &str, line: usize) -> String {
    let text = std::fs::read_to_string(root.join(file)).unwrap_or_default();
    text.lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim()
        .to_string()
}
