mod collisions;
mod prepared;
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
fn prepared_catalog_keeps_symlinked_queue_producers_and_workers_in_one_namespace() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/symlink-workspace/link");
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let files = crate::codebase::ts_source::discover_files_from_visible(&root, &[], &visible_paths);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            queue: true,
            queue_factory_names: vec!["createQueue".to_string()],
            ..Default::default()
        },
    );
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::from_visible(&root, &[], &visible_paths);
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let report = analyze_project_with_prepared_facts_and_catalog_and_session(
        &root,
        &catalog,
        &[],
        &facts,
        &session,
    )
    .unwrap();

    assert!(
        report.producers.iter().any(|producer| {
            producer.file.ends_with("/src/producer.ts")
                && producer
                    .queue_file
                    .as_deref()
                    .is_some_and(|path| path.ends_with("/src/queues.ts"))
                && producer.queue_name.as_deref() == Some("emails")
        }),
        "{report:#?}"
    );
    assert!(
        report.workers.iter().any(|worker| {
            worker.file.ends_with("/src/worker.ts")
                && worker
                    .queue_file
                    .as_deref()
                    .is_some_and(|path| path.ends_with("/src/queues.ts"))
                && worker
                    .processor_file
                    .as_deref()
                    .is_some_and(|path| path.ends_with("/src/processors.ts"))
        }),
        "{report:#?}"
    );
}

#[test]
fn prepared_catalog_indexed_queue_report_matches_plain_report_for_package_aliases() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-tsconfig-catalog");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let files = crate::codebase::ts_source::discover_files_from_visible(&root, &[], &visible_paths);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            queue: true,
            ..Default::default()
        },
    );
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::from_visible(&root, &[], &visible_paths);
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());

    let plain = analyze_project_with_prepared_facts_and_catalog_and_session(
        &root,
        &catalog,
        &[],
        &facts,
        &session,
    )
    .unwrap();
    let indexed = analyze_project_with_prepared_facts_indexed_and_catalog_and_session(
        &root,
        &catalog,
        &[],
        &facts,
        &session,
    )
    .unwrap();

    let mut plain_producers = plain.producers.clone();
    let mut indexed_producers = indexed.report().producers.clone();
    plain_producers.sort();
    indexed_producers.sort();
    assert_eq!(indexed_producers, plain_producers);
    let mut plain_workers = plain.workers.clone();
    let mut indexed_workers = indexed.report().workers.clone();
    plain_workers.sort();
    indexed_workers.sort();
    assert_eq!(indexed_workers, plain_workers);
    assert_eq!(indexed.report().jobs, plain.jobs);
    assert_eq!(indexed.report().edges, plain.edges);
    assert_eq!(indexed.report().diagnostics, plain.diagnostics);
    assert_eq!(indexed.report().check, plain.check);
    assert!(plain.check.is_empty(), "{plain:#?}");
    for (package, queue) in [("a", "a-queue"), ("b", "b-queue")] {
        let file = format!("packages/{package}/src/enqueue.ts");
        let queue_file = format!("packages/{package}/src/queues/email.ts");
        assert!(
            plain.producers.iter().any(|producer| {
                producer.file == file
                    && producer.queue_file.as_deref() == Some(&queue_file)
                    && producer.queue_name.as_deref() == Some(queue)
            }),
            "{plain:#?}"
        );
    }
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
    use crate::codebase::check_facts::{
        collect_file_facts_with_session_and_sources, CheckFactPlan,
    };

    let root = fixture("custom-factory");
    let queue_file = root.join("queues/notifications.ts");
    let plan = CheckFactPlan {
        queue: true,
        queue_factory_names: vec!["createQueue".to_string()],
        ..Default::default()
    };
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
        std::slice::from_ref(&queue_file),
    ));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    let facts = collect_file_facts_with_session_and_sources(
        &session,
        &root,
        &queue_file,
        &plan,
        None,
        &sources,
    )
    .expect("should collect facts for queue file");
    let queue_facts = facts
        .ts
        .queue_project
        .clone()
        .expect("queue facts should be present");
    assert_eq!(
        queue_facts.queue_exports.get("notificationsQueue"),
        Some(&"notifications".to_string()),
        "check-mode should detect factory-created queue bindings when factory names are configured"
    );
}

mod projection;
