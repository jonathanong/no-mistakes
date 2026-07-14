use crate::queue::{analyze_project, analyze_project_indexed, related, RelatedDirection};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/queue/colliding-job-names")
}

#[test]
fn indexed_traversal_expands_colliding_jobs_reached_transitively() {
    // Distinct queue names intentionally collapse to the same public `queues.ts#send` node.
    // Starting outside that node verifies aliases also expand after traversal begins.
    let plain = analyze_project(&fixture(), None, &[]).unwrap();
    let indexed = analyze_project_indexed(&fixture(), None, &[]).unwrap();
    assert_eq!(indexed.report().edges, plain.edges);

    let producer_roots = vec!["producer-alpha.ts".to_string()];
    for depth in [Some(1), Some(2), None] {
        assert_eq!(
            indexed.edge_view(&producer_roots, depth),
            crate::cli::edge_view(&plain.edges, &producer_roots, depth),
        );
    }
    let depth_two = indexed.edge_view(&producer_roots, Some(2));
    assert!(depth_two.iter().any(|edge| edge.to == "worker-alpha.ts"));
    assert!(depth_two.iter().any(|edge| edge.to == "worker-beta.ts"));

    let worker_roots = vec!["worker-alpha.ts".to_string()];
    let dependents = indexed.related(&worker_roots, RelatedDirection::Dependents);
    assert_eq!(
        dependents,
        related(&plain, &worker_roots, RelatedDirection::Dependents),
    );
    assert!(dependents.iter().any(|edge| edge.to == "producer-alpha.ts"));
    assert!(dependents.iter().any(|edge| edge.to == "producer-beta.ts"));
}
