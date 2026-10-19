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
            if let Some(lines) = unresolved.get(&queue) {
                out.push(unresolved_worker(root, worker, &job, lines));
            } else {
                out.push(worker.unmatched(root, &job));
            }
        }
    }
    out
}

fn unresolved_locations(
    root: &Path,
    producers: &[InternalProducer],
) -> HashMap<QueueKey, Vec<String>> {
    let mut out: HashMap<QueueKey, Vec<String>> = HashMap::new();
    for producer in producers {
        if producer.site.job.is_some() {
            continue;
        }
        let Some(queue) = &producer.queue else {
            continue;
        };
        out.entry(queue.clone()).or_default().push(format!(
            "{}:{}",
            relative_string(root, &producer.site.file),
            producer.site.line
        ));
    }
    for lines in out.values_mut() {
        lines.sort();
        lines.dedup();
    }
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
