use super::common::{file_paths, run_json};
use std::path::PathBuf;

#[test]
fn wrapper_only_queue_anchor_coexists_with_file_queries() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/queue-followups/fixture");
    let anchor = "queue:opaque#welcome";
    for (direction, kind, endpoint) in [
        ("deps", "queue-worker", "worker.ts"),
        ("dependents", "queue-enqueue", "enqueue.ts"),
    ] {
        let edges = run_json(
            &root,
            &["queues", "related", anchor, "--direction", direction],
        );
        assert!(
            edges.as_array().unwrap().iter().any(|edge| {
                edge["from"] == anchor && edge["to"] == endpoint && edge["kind"] == kind
            }),
            "{direction}: {edges}"
        );
    }

    let anchor_dependents = run_json(&root, &["dependents", anchor]);
    assert_eq!(anchor_dependents["roots"], serde_json::json!([anchor]));
    assert!(file_paths(&anchor_dependents).is_empty());
    assert_eq!(anchor_dependents["diagnostics"], serde_json::json!([]));
    assert_eq!(run_json(&root, &["related", anchor]), anchor_dependents);

    let anchor_plan = run_json(
        &root,
        &["tests", "plan", "vitest", "--changed-file", anchor],
    );
    assert_eq!(anchor_plan["changed_files"], serde_json::json!([anchor]));
    assert_eq!(anchor_plan["selected_tests"], serde_json::json!([]));
    assert_eq!(anchor_plan["warnings"], serde_json::json!([]));
    assert_eq!(anchor_plan["fallback_triggered"], false);

    let dependents = run_json(&root, &["dependents", "worker.ts"]);
    assert_eq!(file_paths(&dependents), vec!["worker.test.ts"]);
    assert_eq!(run_json(&root, &["related", "worker.ts"]), dependents);

    let plan = run_json(
        &root,
        &["tests", "plan", "vitest", "--changed-file", "worker.ts"],
    );
    assert_eq!(plan["changed_files"], serde_json::json!(["worker.ts"]));
    assert_eq!(plan["fallback_triggered"], false);
    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 1, "{plan}");
    assert_eq!(selected[0]["test_file"], "worker.test.ts");
}
