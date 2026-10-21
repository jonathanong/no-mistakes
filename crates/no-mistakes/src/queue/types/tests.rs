use super::EdgeKind;

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
