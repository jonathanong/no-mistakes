use crate::edge_index::{CanonicalEdge, EdgeIndex};
use crate::queue::extract::FileFacts;
use crate::queue::graph_model::{
    dedup_sorted, diagnostics, node_name, InternalProducer, InternalWorker, PreparedProjectReport,
    ProjectReport,
};
use crate::queue::source::relative_string;
use crate::queue::types::{
    Edge, EdgeKind, JobKey, QueueJobNode, QueueKey, RelationshipEdge, RelationshipNode,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub(super) fn build_report(
    root: &Path,
    producers: Vec<InternalProducer>,
    workers: Vec<InternalWorker>,
    facts: &HashMap<PathBuf, FileFacts>,
) -> ProjectReport {
    build_report_and_relationships(root, producers, workers, facts).0
}

pub(super) fn build_prepared_report(
    root: &Path,
    producers: Vec<InternalProducer>,
    workers: Vec<InternalWorker>,
    facts: &HashMap<PathBuf, FileFacts>,
) -> PreparedProjectReport {
    let (report, mut relationships) =
        build_report_and_relationships(root, producers, workers, facts);
    relationships.sort_by_key(|edge| {
        (
            public_node(root, &edge.from),
            public_node(root, &edge.to),
            edge.kind,
        )
    });
    relationships.dedup();
    let mut nodes_by_name = HashMap::<String, Vec<RelationshipNode>>::new();
    for relationship in &relationships {
        for node in [&relationship.from, &relationship.to] {
            let nodes = nodes_by_name.entry(public_node(root, node)).or_default();
            if !nodes.contains(node) {
                nodes.push(node.clone());
            }
        }
    }
    for nodes in nodes_by_name.values_mut() {
        nodes.sort();
    }
    let index = EdgeIndex::from_edges(
        relationships
            .into_iter()
            .map(|edge| CanonicalEdge::new(edge.from, edge.to, edge.kind)),
    );
    PreparedProjectReport {
        root: root.to_path_buf(),
        report,
        index,
        nodes_by_name,
    }
}

fn build_report_and_relationships(
    root: &Path,
    producers: Vec<InternalProducer>,
    workers: Vec<InternalWorker>,
    facts: &HashMap<PathBuf, FileFacts>,
) -> (ProjectReport, Vec<RelationshipEdge>) {
    let producer_index = index_producers(&producers);
    let worker_index = index_workers(&workers);
    let wildcards = wildcard_queues(&workers);
    let mut relationships = Vec::new();
    let mut check = Vec::new();
    for (job, producers_for_job) in &producer_index {
        let workers_for_job = worker_index.get(job).cloned().unwrap_or_default();
        if workers_for_job.is_empty() && !wildcards.contains(&queue_key(job)) {
            check.extend(
                producers_for_job
                    .iter()
                    .map(|producer| producer.unmatched(root)),
            );
            continue;
        }
        add_matched_job(job, producers_for_job, &workers_for_job, &mut relationships);
    }
    for worker in &workers {
        for (job, _) in worker.job_keys() {
            if !producer_index.contains_key(&job) {
                check.push(worker.unmatched(root, &job));
            }
        }
    }
    let report = ProjectReport {
        producers: producers.iter().map(|p| p.public(root)).collect(),
        workers: workers.iter().map(|w| w.public(root)).collect(),
        jobs: public_jobs(root, &relationships),
        edges: public_edges(root, &relationships),
        diagnostics: diagnostics(root, facts, &producers, &workers),
        check: dedup_sorted(check),
    };
    (report, relationships)
}

fn index_producers(producers: &[InternalProducer]) -> HashMap<JobKey, Vec<&InternalProducer>> {
    producers
        .iter()
        .filter_map(|p| p.job_key())
        .fold(HashMap::new(), |mut map, producer| {
            map.entry(producer.0)
                .or_insert_with(Vec::new)
                .push(producer.1);
            map
        })
}

fn index_workers(workers: &[InternalWorker]) -> HashMap<JobKey, Vec<&InternalWorker>> {
    workers
        .iter()
        .flat_map(InternalWorker::job_keys)
        .fold(HashMap::new(), |mut map, worker| {
            map.entry(worker.0).or_insert_with(Vec::new).push(worker.1);
            map
        })
}

fn wildcard_queues(workers: &[InternalWorker]) -> HashSet<QueueKey> {
    workers
        .iter()
        .filter(|w| w.site.wildcard)
        .filter_map(|w| w.queue.clone())
        .collect()
}

fn add_matched_job(
    job: &JobKey,
    producers: &[&InternalProducer],
    workers: &[&InternalWorker],
    relationships: &mut Vec<RelationshipEdge>,
) {
    let node = RelationshipNode::Job(job.clone());
    relationships.extend(producers.iter().map(|producer| RelationshipEdge {
        from: RelationshipNode::File(producer.site.file.clone()),
        to: node.clone(),
        kind: EdgeKind::QueueEnqueue,
    }));
    relationships.extend(workers.iter().map(|worker| {
        RelationshipEdge {
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
        }
    }));
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

fn queue_key(job: &JobKey) -> QueueKey {
    QueueKey {
        queue_file: job.queue_file.clone(),
        queue_name: job.queue_name.clone(),
    }
}
