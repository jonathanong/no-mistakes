use crate::edge_index::{CanonicalEdge, PreparedRelationshipIndex};
use crate::queue::extract::FileFacts;
use crate::queue::graph_model::{
    dedup_sorted, diagnostics, node_name, InternalProducer, InternalWorker, PreparedProjectReport,
    ProjectReport,
};
use crate::queue::source::relative_string;
use crate::queue::test_scope::ConfiguredTestScope;
use crate::queue::types::{
    Edge, EdgeKind, JobKey, QueueJobNode, RelationshipEdge, RelationshipNode,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[path = "graph_checks.rs"]
mod checks;
pub(super) fn build_report(
    root: &Path,
    producers: Vec<InternalProducer>,
    workers: Vec<InternalWorker>,
    facts: &HashMap<PathBuf, FileFacts>,
    test_scope: &ConfiguredTestScope,
) -> ProjectReport {
    build_report_and_relationships(root, producers, workers, facts, test_scope).0
}

pub(super) fn build_prepared_report(
    root: &Path,
    producers: Vec<InternalProducer>,
    workers: Vec<InternalWorker>,
    facts: &HashMap<PathBuf, FileFacts>,
    test_scope: &ConfiguredTestScope,
) -> PreparedProjectReport {
    let (report, relationships) =
        build_report_and_relationships(root, producers, workers, facts, test_scope);
    let relationships = PreparedRelationshipIndex::from_edges(
        relationships
            .into_iter()
            .map(|edge| CanonicalEdge::new(edge.from, edge.to, edge.kind)),
        |node| public_node(root, node),
    );
    PreparedProjectReport {
        report,
        relationships,
    }
}

fn build_report_and_relationships(
    root: &Path,
    producers: Vec<InternalProducer>,
    workers: Vec<InternalWorker>,
    facts: &HashMap<PathBuf, FileFacts>,
    test_scope: &ConfiguredTestScope,
) -> (ProjectReport, Vec<RelationshipEdge>) {
    let producer_index = checks::index_producers(&producers);
    let worker_index = checks::index_workers(&workers);
    let wildcards = checks::wildcard_queues(&workers);
    let mut relationships = Vec::new();
    let mut check = Vec::new();
    let mut report_diagnostics = diagnostics(root, facts, &producers, &workers, test_scope);
    for (job, producers_for_job) in &producer_index {
        let workers_for_job = worker_index.get(job).cloned().unwrap_or_default();
        if workers_for_job.is_empty() && !wildcards.contains(&checks::queue_key(job)) {
            checks::record_unmatched_producers(
                root,
                job,
                producers_for_job,
                &workers,
                test_scope,
                &mut check,
                &mut report_diagnostics,
            );
            continue;
        }
        add_matched_job(
            root,
            job,
            producers_for_job,
            &workers_for_job,
            test_scope,
            &mut relationships,
        );
    }
    check.extend(checks::unmatched_workers(
        root,
        &workers,
        &producer_index,
        &producers,
        test_scope,
    ));
    let (job_state_checks, job_state_diagnostics) =
        crate::queue::job_state::findings_and_diagnostics(root, facts, test_scope);
    check.extend(job_state_checks);
    check.extend(crate::queue::job_options::local_checks(
        root, facts, test_scope,
    ));
    check.extend(crate::queue::cycles::queue_cycles(
        root, &producers, &workers, test_scope,
    ));
    let (dedup_checks, dedup_diagnostics) =
        crate::queue::rate_limit::findings(root, &producers, &workers, facts, test_scope);
    check.extend(dedup_checks);
    let mut check = dedup_sorted(check);
    check.extend(crate::queue::priority_mix::mixed_priority_findings(
        root,
        &producers,
        facts,
        test_scope,
        &mut report_diagnostics,
    ));
    check.sort();
    report_diagnostics.extend(job_state_diagnostics);
    report_diagnostics.extend(dedup_diagnostics);
    let mut report = ProjectReport {
        producers: producers.iter().map(|p| p.public(root)).collect(),
        workers: workers.iter().map(|w| w.public(root)).collect(),
        jobs: public_jobs(root, &relationships),
        edges: public_edges(root, &relationships),
        diagnostics: dedup_sorted(report_diagnostics),
        check,
    };
    report.producers.sort();
    report.workers.sort();
    (report, relationships)
}

fn add_matched_job(
    root: &Path,
    job: &JobKey,
    producers: &[&InternalProducer],
    workers: &[&InternalWorker],
    test_scope: &ConfiguredTestScope,
    relationships: &mut Vec<RelationshipEdge>,
) {
    let node = RelationshipNode::Job(job.clone());
    relationships.extend(
        producers
            .iter()
            .filter(|producer| !test_scope.is_match(root, &producer.site.file))
            .map(|producer| RelationshipEdge {
                from: RelationshipNode::File(producer.site.file.clone()),
                to: node.clone(),
                kind: EdgeKind::QueueEnqueue,
            }),
    );
    relationships.extend(
        workers
            .iter()
            .filter(|worker| !test_scope.is_match(root, &worker.site.file))
            .map(|worker| RelationshipEdge {
                from: node.clone(),
                to: RelationshipNode::File(
                    worker
                        .site
                        .processor_file
                        .as_ref()
                        .unwrap_or(&worker.site.file)
                        .clone(),
                ),
                kind: EdgeKind::QueueWorker,
            }),
    );
}

fn public_jobs(root: &Path, relationships: &[RelationshipEdge]) -> Vec<QueueJobNode> {
    dedup_sorted(
        relationships
            .iter()
            .flat_map(|edge| [&edge.from, &edge.to])
            .filter_map(|node| match node {
                RelationshipNode::File(_) => None,
                RelationshipNode::Job(job) => Some(QueueJobNode {
                    queue_file: relative_string(root, &job.queue_file),
                    queue_name: job.queue_name.clone(),
                    job: job.job.clone(),
                }),
            })
            .collect(),
    )
}

fn public_edges(root: &Path, relationships: &[RelationshipEdge]) -> Vec<Edge> {
    dedup_sorted(
        relationships
            .iter()
            .map(|edge| Edge {
                from: public_node(root, &edge.from),
                to: public_node(root, &edge.to),
                kind: edge.kind,
            })
            .collect(),
    )
}

pub(crate) fn public_node(root: &Path, node: &RelationshipNode) -> String {
    match node {
        RelationshipNode::File(file) => relative_string(root, file),
        RelationshipNode::Job(job) => node_name(root, job),
    }
}
