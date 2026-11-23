use crate::queue::graph_model::{CheckFinding, InternalProducer, InternalWorker};
use crate::queue::source::relative_string;
use crate::queue::types::{JobKey, QueueKey};
use std::collections::HashMap;
use std::path::Path;

pub(super) fn unmatched_workers(
    root: &Path,
    workers: &[InternalWorker],
    producers_for_job: &HashMap<JobKey, Vec<&InternalProducer>>,
    producers: &[InternalProducer],
) -> Vec<CheckFinding> {
    let unresolved = unresolved_locations(root, producers);
    let mut out = Vec::new();
    for worker in workers {
        for (job, _) in worker.job_keys() {
            if producers_for_job.contains_key(&job) {
                continue;
            }
            let queue = QueueKey {
                queue_file: job.queue_file.clone(),
                queue_name: job.queue_name.clone(),
            };
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
