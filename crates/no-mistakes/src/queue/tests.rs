mod resolver;
use super::*;
use crate::queue::extract_model::FileFacts;
use crate::queue::graph_model::{diagnostics, ProjectReport};
use crate::queue::types::{Edge, EdgeKind};
use resolver::extract_file_with_factories;
use std::collections::HashMap;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop")
        .join(name)
        .join("fixture")
}

#[test]
fn basic_project_reports_queue_edges() {
    let report = analyze_project(&fixture("basic"), None, &[]).unwrap();
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
}

#[test]
fn dynamic_producer_is_warning_not_check_failure() {
    let report = analyze_project(&fixture("dynamic"), None, &[]).unwrap();
    assert!(report
        .check
        .iter()
        .any(|finding| finding.kind == "unmatched-worker"));
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("dynamic")));
}

#[test]
fn flow_producer_edges_are_supported() {
    let report = analyze_project(&fixture("flow"), None, &[]).unwrap();
    assert_eq!(report.check, vec![]);
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.from == "flow.ts" && edge.to == "flow.ts#resize"));
    assert!(report
        .producers
        .iter()
        .any(|producer| producer.job.as_deref() == Some("resize")
            && producer.raw_job.as_deref() == Some("JOB")));
}

#[test]
fn tsconfig_paths_resolve_queue_imports() {
    let root = fixture("tsconfig-paths");
    let report = analyze_project(&root, Some(&root.join("tsconfig.json")), &[]).unwrap();
    assert_eq!(report.check, vec![]);
    assert!(report
        .producers
        .iter()
        .any(|producer| producer.queue_name.as_deref() == Some("email-paths")));
}

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

#[test]
fn add_bulk_and_wildcard_worker_are_supported() {
    let report = analyze_project(&fixture("bulk"), None, &[]).unwrap();
    assert_eq!(report.check, vec![]);
    assert_eq!(report.jobs.len(), 2);
    assert!(report.workers.iter().any(|worker| worker.wildcard));
}

#[test]
fn unmatched_static_producer_and_worker_are_check_findings() {
    let report = analyze_project(&fixture("unmatched"), None, &[]).unwrap();
    assert!(report
        .check
        .iter()
        .any(|finding| finding.kind == "unmatched-producer"));
    assert!(report
        .check
        .iter()
        .any(|finding| finding.kind == "unmatched-worker"));
}

#[test]
fn filters_limit_discovered_sources() {
    let report = analyze_project(&fixture("basic"), None, &["enqueue.ts".to_string()]).unwrap();
    assert!(report.edges.is_empty());
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("producer")));
}

#[test]
fn missing_tsconfig_returns_error() {
    let err = analyze_project(
        &fixture("basic"),
        Some(&fixture("basic").join("missing.json")),
        &[],
    )
    .unwrap_err();
    let context = err.to_string();
    assert!(
        context.contains("loading tsconfig") && context.contains("missing.json"),
        "{context}"
    );
    let chain = format!("{err:#}");
    assert!(
        chain.contains("No such file") || chain.contains("os error"),
        "{chain}"
    );
}

#[test]
fn alternate_syntaxes_and_dynamic_sites_are_recorded() {
    let root = fixture("syntax");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("producer")));
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("worker")));
    assert!(report
        .workers
        .iter()
        .any(|worker| worker.processor_file.as_deref() == Some("processor.ts")));
    assert!(report
        .producers
        .iter()
        .any(|producer| producer.raw_job.as_deref() == Some("JOB")));
    assert!(report
        .producers
        .iter()
        .any(|producer| producer.raw_job.as_deref() == Some("DYNAMIC_JOB")));
}

#[test]
fn diagnostics_include_parser_warnings_from_facts() {
    let root = fixture("basic");
    let path = root.join("enqueue.ts");
    let mut facts = FileFacts::default();
    facts.diagnostics.push((7, "synthetic warning".to_string()));
    let diagnostics = diagnostics(&root, &HashMap::from([(path, facts)]), &[], &[]);
    assert_eq!(diagnostics[0].line, 7);
    assert_eq!(diagnostics[0].message, "synthetic warning");
}

#[test]
fn missing_root_with_shared_facts_returns_empty_report() {
    let root = fixture("does-not-exist");
    let facts = crate::codebase::check_facts::CheckFactMap::default();
    let report = analyze_project_with_facts(&root, None, &[], &facts).unwrap();
    assert!(report.edges.is_empty());
    assert!(report.producers.is_empty());
    assert!(report.workers.is_empty());
}

#[test]
fn shared_facts_filter_excludes_non_matching_files() {
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

    // Filter to only worker.ts so that enqueue.ts and queues.ts are excluded.
    // This causes the `continue` branch inside the filter block (graph.rs lines 62-64)
    // to execute for each skipped file.
    let report =
        analyze_project_with_facts(&root, None, &["worker.ts".to_string()], &facts).unwrap();

    // With no queue definitions visible (queues.ts was filtered out) the worker cannot
    // resolve its queue, so job_keys() returns empty and no edges or check findings appear.
    assert!(report.edges.is_empty());
    assert!(report.check.is_empty());
    // worker.ts itself is still present in the workers list.
    assert!(!report.workers.is_empty());
}

#[test]
fn custom_factory_extraction_detects_queue_export() {
    let root = fixture("custom-factory");
    let queue_file = root.join("queues/notifications.ts");
    let factory_names = vec!["createQueue".to_string()];
    let facts = extract_file_with_factories(&queue_file, &factory_names).unwrap();
    assert_eq!(
        facts.queue_exports.get("notificationsQueue"),
        Some(&"notifications".to_string()),
        "should detect notificationsQueue as a queue binding"
    );
}

#[test]
fn custom_factory_send_notification_has_producer_sites() {
    let root = fixture("custom-factory");
    let send_file = root.join("producers/send-notification.ts");
    let facts = extract_file_with_factories(&send_file, &[]).unwrap();
    assert!(
        !facts.producers.is_empty(),
        "should detect notificationsQueue.add() as producer"
    );
    assert_eq!(facts.producers[0].binding, "notificationsQueue");
}

#[test]
fn custom_factory_from_v2_config_detected_as_queue_definition() {
    let root = fixture("custom-factory");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(
        report.edges.iter().any(|edge| {
            edge.from.contains("send-notification") && edge.to.contains("notifications")
        }),
        "producer should connect to notifications queue, edges: {:?}",
        report.edges
    );
}

#[test]
fn ignored_queue_processor_is_not_resolved_from_disk() {
    let fixture = crate::test_support::materialize_gitignore_fixture("transitive-visibility");

    let report = analyze_project(fixture.path(), None, &[]).unwrap();

    assert!(report
        .workers
        .iter()
        .any(|worker| worker.file == "queue/worker.ts" && worker.processor_file.is_none()));
}

#[test]
fn custom_factory_respected_in_check_mode_shared_facts() {
    use crate::codebase::check_facts::{collect_file_facts, CheckFactPlan};

    let root = fixture("custom-factory");
    let queue_file = root.join("queues/notifications.ts");
    let plan = CheckFactPlan {
        queue: true,
        queue_factory_names: vec!["createQueue".to_string()],
        ..Default::default()
    };
    let facts = collect_file_facts(&root, &queue_file, &plan, None)
        .expect("should collect facts for queue file");
    let queue_facts = facts
        .ts
        .queue_project
        .expect("queue facts should be present");
    assert_eq!(
        queue_facts.queue_exports.get("notificationsQueue"),
        Some(&"notifications".to_string()),
        "check-mode should detect factory-created queue bindings when factory names are configured"
    );
}

#[test]
fn typed_index_preserves_colliding_file_and_job_nodes() {
    use crate::edge_index::{CanonicalEdge, EdgeIndex};
    use crate::queue::graph_model::PreparedProjectReport;
    use crate::queue::types::{JobKey, RelationshipNode};

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
            ("queues.ts#send".into(), vec![file, job]),
            ("worker.ts".into(), vec![worker]),
        ]),
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
