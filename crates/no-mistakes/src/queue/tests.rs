mod catalog;
mod collisions;
mod debounce_ttl_review;
mod enqueue_wrappers;
mod extract;
mod factory_detectors;
mod factory_false_positives;
mod factory_reexports;
mod factory_roles;
mod job_id;
mod job_options;
mod job_options_flow;
mod job_options_policy;
mod lang;
mod prepared;
mod priority_mix;
mod processor_identifiers;
mod related;
mod resolver;
mod scheduler_template;
pub(crate) mod session_lang;
mod shared_facts;
mod shared_rate_limit;
mod wait_forms;
mod wait_scopes;
mod worker_rate_limit;
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
        .any(|worker| worker.file == "worker.ts" && worker.processor_file.is_none()));
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
    let diagnostics = diagnostics(
        &root,
        &HashMap::from([(path, facts)]),
        &[],
        &[],
        &crate::queue::test_scope::ConfiguredTestScope::empty(),
    );
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
    assert!(matches!(
        &facts.producers[0].binding,
        crate::queue::extract_model::ProducerBinding::Named(name) if name == "notificationsQueue"
    ));
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
    use crate::codebase::check_facts::tests::compatibility::collect_file_facts_with_session_and_sources;
    use crate::codebase::check_facts::CheckFactPlan;

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
        false,
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

mod enqueue_wrapper_exports;
mod job_options_followups;
mod job_retention;
mod job_state;
mod projection;
mod workspace_exports;
