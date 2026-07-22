use super::*;
use crate::edge_index::{CanonicalEdge, EdgeIndex, NodeAliases};
use crate::queue::graph_model::PreparedProjectReport;
use crate::queue::types::{JobKey, RelationshipNode};

#[test]
fn typed_index_preserves_colliding_file_and_job_nodes() {
    let root = PathBuf::from("/repo");
    let file = RelationshipNode::File(root.join("queues.ts#send"));
    let job = RelationshipNode::Job(JobKey {
        queue_file: root.join("queues.ts"),
        queue_name: "queue".into(),
        job: "send".into(),
    });
    let worker = RelationshipNode::File(root.join("worker.ts"));
    let report = PreparedProjectReport {
        root,
        report: ProjectReport {
            producers: vec![],
            workers: vec![],
            jobs: vec![],
            edges: vec![],
            diagnostics: vec![],
            check: vec![],
        },
        index: EdgeIndex::from_edges([
            CanonicalEdge::new(file.clone(), job.clone(), EdgeKind::QueueEnqueue),
            CanonicalEdge::new(job.clone(), worker.clone(), EdgeKind::QueueWorker),
        ]),
        nodes_by_name: HashMap::from([
            ("queues.ts#send".into(), vec![file.clone(), job.clone()]),
            ("worker.ts".into(), vec![worker]),
        ]),
        aliases: NodeAliases::from_groups([vec![file, job]]),
    };

    assert_eq!(
        report.edge_view(&["queues.ts#send".into()], Some(1)),
        vec![
            Edge {
                from: "queues.ts#send".into(),
                to: "queues.ts#send".into(),
                kind: EdgeKind::QueueEnqueue
            },
            Edge {
                from: "queues.ts#send".into(),
                to: "worker.ts".into(),
                kind: EdgeKind::QueueWorker
            },
        ]
    );
}

#[test]
fn indexed_queue_projection_matches_compatibility_projection() {
    let root = fixture("basic");
    let plain = analyze_project(&root, None, &[]).unwrap();
    let indexed = analyze_project_indexed(&root, None, &[]).unwrap();
    assert_eq!(indexed.report().edges, plain.edges);
    let roots = vec!["enqueue.ts".to_string()];
    assert_eq!(
        indexed.edge_view(&roots, None),
        crate::cli::edge_view(&plain.edges, &roots, None)
    );
    assert_eq!(
        indexed.related(&roots, RelatedDirection::Both),
        related(&plain, &roots, RelatedDirection::Both)
    );
}
