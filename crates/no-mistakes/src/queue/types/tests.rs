use super::{Diagnostic, EdgeKind, Severity};

#[test]
fn queue_edge_kind_display_matches_serialized_kebab_names() {
    assert_eq!(EdgeKind::QueueEnqueue.to_string(), "queue-enqueue");
    assert_eq!(EdgeKind::QueueWorker.to_string(), "queue-worker");
    assert_eq!(
        serde_json::to_value(EdgeKind::QueueEnqueue).unwrap(),
        serde_json::json!("queue-enqueue")
    );
    assert_eq!(
        serde_json::to_value(EdgeKind::QueueWorker).unwrap(),
        serde_json::json!("queue-worker")
    );
}

#[test]
fn queue_diagnostic_severity_and_formatters_match_serialized_names() {
    assert_eq!(Severity::Warning.to_string(), "warning");
    assert_eq!(Severity::Error.to_string(), "error");
    assert_eq!(
        serde_json::to_value(Severity::Warning).unwrap(),
        serde_json::json!("warning")
    );
    assert_eq!(
        serde_json::to_value(Severity::Error).unwrap(),
        serde_json::json!("error")
    );
    let warning = Diagnostic {
        severity: Severity::Warning,
        file: "worker.ts".to_string(),
        line: 4,
        message: "unjudged".to_string(),
    };
    assert_eq!(
        crate::queue::format_diagnostic(&warning),
        "warning: worker.ts:4 unjudged"
    );
    assert_eq!(
        crate::queue::format_aggregate_warning(&warning),
        "queues-check: worker.ts:4 unjudged"
    );
}
