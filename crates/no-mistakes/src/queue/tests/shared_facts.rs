use super::*;

#[test]
fn shared_facts_project_reports_queue_edges() {
    let root = fixture("basic");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            queue: true,
            ..Default::default()
        },
    );

    let report = analyze_project_with_facts(&root, None, &[], &facts).unwrap();

    assert_eq!(report.check, vec![]);
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.from == "enqueue.ts" && edge.to == "queues.ts#sendWelcome"));
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.from == "queues.ts#sendWelcome" && edge.to == "worker.ts"));
}

#[test]
fn pass4a_ignored_queue_module_does_not_shadow_visible_producer_fallback() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4a-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());

    let report = analyze_project(fixture.path(), None, &[]).unwrap();

    assert!(report.producers.iter().any(|producer| {
        producer.queue_name.as_deref() == Some("visible-emails")
            && producer.queue_file.as_deref() == Some("queue/queues.ts")
    }));
}

#[test]
fn missing_project_root_returns_empty_report() {
    let report = analyze_project(&fixture("does-not-exist"), None, &[]).unwrap();
    assert!(report.edges.is_empty());
    assert!(report.producers.is_empty());
    assert!(report.workers.is_empty());
    let indexed = analyze_project_indexed(&fixture("does-not-exist"), None, &[]).unwrap();
    assert!(indexed.report.producers.is_empty());
}
