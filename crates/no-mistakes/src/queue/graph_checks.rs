use crate::queue::graph_model::{CheckFinding, InternalProducer, InternalWorker};
use crate::queue::source::relative_string;
use crate::queue::test_scope::ConfiguredTestScope;
use crate::queue::types::{Diagnostic, JobKey, QueueKey, Severity};
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(super) fn index_producers(
    producers: &[InternalProducer],
) -> HashMap<JobKey, Vec<&InternalProducer>> {
    producers
        .iter()
        .filter_map(|p| p.job_key())
        .fold(HashMap::new(), |mut map, producer| {
            map.entry(producer.0).or_default().push(producer.1);
            map
        })
}

pub(super) fn index_workers(workers: &[InternalWorker]) -> HashMap<JobKey, Vec<&InternalWorker>> {
    workers
        .iter()
        .flat_map(InternalWorker::job_keys)
        .fold(HashMap::new(), |mut map, worker| {
            map.entry(worker.0).or_default().push(worker.1);
            map
        })
}

pub(super) fn queue_key(job: &JobKey) -> QueueKey {
    QueueKey {
        queue_file: job.queue_file.clone(),
        queue_name: job.queue_name.clone(),
    }
}

pub(super) fn wildcard_queues(workers: &[InternalWorker]) -> HashSet<QueueKey> {
    workers
        .iter()
        .filter(|w| w.site.wildcard)
        .filter_map(|w| w.queue.clone())
        .collect()
}

pub(super) fn record_unmatched_producers(
    root: &Path,
    job: &JobKey,
    producers_for_job: &[&InternalProducer],
    workers: &[InternalWorker],
    test_scope: &ConfiguredTestScope,
    check: &mut Vec<CheckFinding>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let blocking_workers = unresolved_workers_for_job(root, workers, job);
    for producer in producers_for_job {
        if test_scope.is_match(root, &producer.site.file) {
            continue;
        }
        if blocking_workers.is_empty() {
            check.push(producer.unmatched(root));
        } else {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                file: relative_string(root, &producer.site.file),
                line: producer.site.line,
                message: format!(
                    "unmatched-producer was not judged because unresolved workers at {} could handle this queue",
                    blocking_workers.join(", ")
                ),
            });
        }
    }
}

fn unresolved_workers_for_job(
    root: &Path,
    workers: &[InternalWorker],
    job: &JobKey,
) -> Vec<String> {
    let mut lines = workers
        .iter()
        .filter(|worker| {
            worker.queue.is_none()
                && worker.site.queue_name.is_none()
                && (worker.site.wildcard
                    || worker.site.jobs.iter().any(|handled| handled == &job.job))
                && (worker.site.queue_name_import.is_some()
                    || worker.reachable_files.contains(&job.queue_file))
        })
        .map(|worker| {
            format!(
                "{}:{}",
                relative_string(root, &worker.site.file),
                worker.site.line
            )
        })
        .collect::<Vec<_>>();
    lines.sort();
    lines.dedup();
    lines
}

pub(super) fn unmatched_workers(
    root: &Path,
    workers: &[InternalWorker],
    producers_for_job: &HashMap<JobKey, Vec<&InternalProducer>>,
    producers: &[InternalProducer],
    test_scope: &ConfiguredTestScope,
) -> Vec<CheckFinding> {
    let unresolved = unresolved_locations(root, producers);
    let mut out = Vec::new();
    for worker in workers {
        if test_scope.is_match(root, &worker.site.file) {
            continue;
        }
        for (job, _) in worker.job_keys() {
            if producers_for_job.contains_key(&job) {
                continue;
            }
            let queue = queue_key(&job);
            let mut lines = unresolved.by_queue.get(&queue).cloned().unwrap_or_default();
            if let Some(job_lines) = unresolved.by_job.get(&job.job) {
                lines.extend(job_lines.iter().cloned());
            }
            lines.extend(unresolved.any_queue.iter().cloned());
            lines.sort();
            lines.dedup();
            if lines.is_empty() {
                out.push(worker.unmatched(root, &job));
            } else {
                out.push(unresolved_worker(root, worker, &job, &lines));
            }
        }
    }
    out
}

struct UnresolvedLocations {
    by_queue: HashMap<QueueKey, Vec<String>>,
    by_job: HashMap<String, Vec<String>>,
    any_queue: Vec<String>,
}

fn unresolved_locations(root: &Path, producers: &[InternalProducer]) -> UnresolvedLocations {
    let mut out = UnresolvedLocations {
        by_queue: HashMap::new(),
        by_job: HashMap::new(),
        any_queue: Vec::new(),
    };
    for producer in producers {
        let line = format!(
            "{}:{}",
            relative_string(root, &producer.site.file),
            producer.site.line
        );
        match (&producer.queue, producer.site.job.as_deref()) {
            (Some(queue), None) => out.by_queue.entry(queue.clone()).or_default().push(line),
            // An unresolved queue may feed this job on any queue definition.
            (None, Some(job)) => out.by_job.entry(job.to_string()).or_default().push(line),
            (None, None) => out.any_queue.push(line),
            (Some(_), Some(_)) => {}
        }
    }
    for lines in out.by_queue.values_mut() {
        lines.sort();
        lines.dedup();
    }
    for lines in out.by_job.values_mut() {
        lines.sort();
        lines.dedup();
    }
    out.any_queue.sort();
    out.any_queue.dedup();
    out
}

fn unresolved_worker(
    root: &Path,
    worker: &InternalWorker,
    job: &JobKey,
    lines: &[String],
) -> CheckFinding {
    let mut finding = worker.unmatched(root, job);
    finding.kind = "unmatched-worker-unresolved-producers".to_string();
    finding.message = format!(
        "unresolved producers at {}; the handler is not known to be dead",
        lines.join(", ")
    );
    finding
}
