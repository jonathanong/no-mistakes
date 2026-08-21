use crate::queue::{analyze_project, EdgeKind};
use std::path::PathBuf;

fn lang_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lang-frontends")
        .join(name)
}

fn ts_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop")
        .join(name)
        .join("fixture")
}

#[test]
fn language_packages_do_not_change_typescript_queue_edges() {
    let off = analyze_project(&ts_fixture("basic"), None, &[]).unwrap();
    let on = analyze_project(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/server-queues-lang/queue-ts-with-lang"),
        None,
        &[],
    )
    .unwrap();
    assert_eq!(off.edges, on.edges);
    assert_eq!(off.producers, on.producers);
    assert_eq!(off.workers, on.workers);
}

#[test]
fn celery_fixture_projects_queue_edges() {
    let report = analyze_project(&lang_fixture("python-celery-django"), None, &[]).unwrap();
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.kind == EdgeKind::QueueEnqueue));
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.kind == EdgeKind::QueueWorker));
}

#[test]
fn celery_filter_excludes_enqueue_file() {
    let report = analyze_project(
        &lang_fixture("python-celery-django"),
        None,
        &["app/tasks.py".into()],
    )
    .unwrap();
    assert!(report
        .edges
        .iter()
        .all(|edge| edge.kind != EdgeKind::QueueEnqueue || !edge.from.ends_with("enqueue.py")));
}

#[test]
fn kafka_filter_skips_unselected_topic_files() {
    let report =
        analyze_project(&lang_fixture("kafka-topics"), None, &["producer.py".into()]).unwrap();
    assert!(report.edges.is_empty());
}

#[test]
fn kafka_fixture_projects_topic_edges() {
    let report = analyze_project(&lang_fixture("kafka-topics"), None, &[]).unwrap();
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.kind == EdgeKind::QueueEnqueue));
}

#[test]
fn symfony_fixture_projects_messenger_edges() {
    let report = analyze_project(&lang_fixture("php-symfony"), None, &[]).unwrap();
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.kind == EdgeKind::QueueEnqueue || edge.kind == EdgeKind::QueueWorker));
}

#[test]
fn rails_sidekiq_fixture_projects_queue_edges() {
    let report = analyze_project(&lang_fixture("rails-sidekiq"), None, &[]).unwrap();
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.kind == EdgeKind::QueueEnqueue));
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.kind == EdgeKind::QueueWorker));
}
