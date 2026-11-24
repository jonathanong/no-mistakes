use crate::queue::cycles::self_dedup_findings;
use crate::queue::extract::FileFacts;
use crate::queue::graph_model::{CheckFinding, InternalProducer, InternalWorker};
use crate::queue::source::relative_string;
use crate::queue::types::{Diagnostic, QueueKey, Severity};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

pub(super) fn findings(
    root: &Path,
    producers: &[InternalProducer],
    workers: &[InternalWorker],
    facts: &HashMap<PathBuf, FileFacts>,
) -> (Vec<CheckFinding>, Vec<Diagnostic>) {
    let (mut checks, mut diagnostics) = self_dedup_findings(root, producers, workers);
    let (rate_checks, rate_diagnostics) = shared_rate_limits(root, producers, workers, facts);
    checks.extend(rate_checks);
    diagnostics.extend(rate_diagnostics);
    let (worker_checks, worker_diagnostics) =
        crate::queue::worker_rate_limit::findings(root, workers, facts);
    checks.extend(worker_checks);
    diagnostics.extend(worker_diagnostics);
    (checks, diagnostics)
}

fn shared_rate_limits(
    root: &Path,
    producers: &[InternalProducer],
    workers: &[InternalWorker],
    facts: &HashMap<PathBuf, FileFacts>,
) -> (Vec<CheckFinding>, Vec<Diagnostic>) {
    let mut checks = Vec::new();
    let mut diagnostics = Vec::new();
    for (path, file) in facts {
        let file_workers: Vec<_> = workers
            .iter()
            .filter(|worker| worker.site.file == *path)
            .collect();
        for site in &file.rate_limits {
            let worker = file_workers[site.worker_index];
            judge(
                root,
                path,
                site.line,
                worker,
                producers,
                &mut checks,
                &mut diagnostics,
            );
        }
    }
    (checks, diagnostics)
}

fn judge(
    root: &Path,
    path: &Path,
    line: usize,
    worker: &InternalWorker,
    producers: &[InternalProducer],
    checks: &mut Vec<CheckFinding>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(queue) = &worker.queue else {
        diagnostics.push(abstain(
            root,
            path,
            line,
            "the worker queue did not resolve, so this rate limit was not judged",
        ));
        return;
    };
    let queue_name = &queue.queue_name;
    match job_names(queue, producers) {
        JobNames::Unknown => {
            let message = format!(
                "the set of job names on `{queue_name}` is unknown, so this rate limit was not judged"
            );
            diagnostics.push(abstain(root, path, line, &message));
        }
        JobNames::Known(names) if names.len() > 1 => {
            let jobs = names.into_iter().collect::<Vec<_>>().join(", ");
            checks.push(CheckFinding {
                kind: "shared-queue-rate-limit".to_string(),
                file: relative_string(root, path),
                line,
                queue_file: Some(relative_string(root, &queue.queue_file)),
                queue_name: Some(queue_name.clone()),
                job: None,
                message: format!(
                    "`{queue_name}` carries more than one job ({jobs}), so this rate limit stalls every job on that queue. Move that job kind to its own queue, or use per-group rate limits."
                ),
            });
        }
        JobNames::Known(_) => {}
    }
}

enum JobNames {
    Unknown,
    Known(BTreeSet<String>),
}

fn job_names(queue: &QueueKey, producers: &[InternalProducer]) -> JobNames {
    let mut names = BTreeSet::new();
    let mut unknown = false;
    for producer in producers {
        if producer.queue.as_ref() != Some(queue) {
            continue;
        }
        match &producer.site.job {
            Some(job) => {
                names.insert(job.clone());
            }
            None => unknown = true,
        }
    }
    if unknown {
        JobNames::Unknown
    } else {
        JobNames::Known(names)
    }
}

fn abstain(root: &Path, path: &Path, line: usize, message: &str) -> Diagnostic {
    Diagnostic {
        severity: Severity::Warning,
        file: relative_string(root, path),
        line,
        message: message.to_string(),
    }
}
