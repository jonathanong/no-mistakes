use super::analyze_project;
use std::path::PathBuf;

#[test]
fn hoisted_flow_cycle_stops_at_the_repeated_child() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-lexical-and-flow");
    let report = analyze_project(&root, None, &[]).unwrap();
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.file == "flow.ts"
                && diagnostic
                    .message
                    .starts_with("job options were not judged")
        })
        .map(|diagnostic| (diagnostic.line, diagnostic.message.as_str()))
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "explicit opts after a cyclic spread are judged: {diagnostics:?}"
    );
}

#[test]
fn cyclic_queue_defaults_are_folded_from_each_spread() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-lexical-and-flow");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(!report.diagnostics.iter().any(|diagnostic| {
        diagnostic.file == "defaults.ts"
            && diagnostic
                .message
                .starts_with("job options were not judged")
    }));
    assert!(report
        .check
        .iter()
        .all(|finding| finding.file != "defaults.ts" || finding.kind != "missing-job-option"));
}
