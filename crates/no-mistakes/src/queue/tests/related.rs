use super::*;

#[test]
fn related_crosses_virtual_queue_jobs() {
    let report = analyze_project(&fixture("basic"), None, &[]).unwrap();
    let edges = related(&report, &["enqueue.ts".to_string()], RelatedDirection::Both);
    assert!(edges.iter().any(|edge| edge.to == "queues.ts#sendWelcome"));
    assert!(edges.iter().any(|edge| edge.to == "worker.ts"));
}

#[test]
fn related_dependents_is_transitive() {
    let report = ProjectReport {
        producers: vec![],
        workers: vec![],
        jobs: vec![],
        edges: vec![
            Edge {
                from: "a.ts".to_string(),
                to: "b.ts".to_string(),
                kind: EdgeKind::QueueEnqueue,
            },
            Edge {
                from: "b.ts".to_string(),
                to: "c.ts".to_string(),
                kind: EdgeKind::QueueEnqueue,
            },
        ],
        diagnostics: vec![],
        check: vec![],
    };
    let edges = related(&report, &["c.ts".to_string()], RelatedDirection::Dependents);
    assert_eq!(edges.len(), 2);
    assert!(edges
        .iter()
        .any(|edge| edge.from == "c.ts" && edge.to == "b.ts"));
    assert!(edges
        .iter()
        .any(|edge| edge.from == "b.ts" && edge.to == "a.ts"));
}

#[test]
fn related_deduplicates_paths_to_seen_nodes() {
    let report = ProjectReport {
        producers: vec![],
        workers: vec![],
        jobs: vec![],
        edges: vec![
            Edge {
                from: "producer.ts".into(),
                to: "shared.ts#sendWelcome".into(),
                kind: EdgeKind::QueueEnqueue,
            },
            Edge {
                from: "alias.ts".into(),
                to: "shared.ts#sendWelcome".into(),
                kind: EdgeKind::QueueWorker,
            },
            Edge {
                from: "shared.ts#sendWelcome".into(),
                to: "worker.ts".into(),
                kind: EdgeKind::QueueWorker,
            },
        ],
        diagnostics: vec![],
        check: vec![],
    };
    let edges = related(
        &report,
        &["producer.ts".into(), "alias.ts".into()],
        RelatedDirection::Deps,
    );
    assert_eq!(
        edges,
        vec![
            Edge {
                from: "alias.ts".into(),
                to: "shared.ts#sendWelcome".into(),
                kind: EdgeKind::QueueWorker
            },
            Edge {
                from: "producer.ts".into(),
                to: "shared.ts#sendWelcome".into(),
                kind: EdgeKind::QueueEnqueue
            },
            Edge {
                from: "shared.ts#sendWelcome".into(),
                to: "worker.ts".into(),
                kind: EdgeKind::QueueWorker
            },
        ]
    );
}
