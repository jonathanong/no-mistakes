use crate::queue::extract::FileFacts;
use crate::queue::extract_model::WorkerRateLimitTarget;
use crate::queue::graph_model::{CheckFinding, InternalWorker};
use crate::queue::source::relative_string;
use crate::queue::types::{Diagnostic, Severity};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[path = "worker_rate_limit_resolve.rs"]
mod resolve_exports;

pub(in crate::queue) use resolve_exports::resolve;

pub(crate) const WORKER_RATE_LIMIT_KIND: &str = "worker-rate-limit-per-process";

const UNRESOLVED_MESSAGE: &str =
    "worker-rate-limit-per-process was not judged because the .rateLimit() receiver did not resolve";

pub(super) fn findings(
    root: &Path,
    workers: &[InternalWorker],
    facts: &HashMap<PathBuf, FileFacts>,
) -> (Vec<CheckFinding>, Vec<Diagnostic>) {
    if facts
        .values()
        .all(|file| file.worker_rate_limits.is_empty())
    {
        return (Vec::new(), Vec::new());
    }
    let workers_by_index = workers_by_file_index(workers);
    let mut checks = Vec::new();
    let mut diagnostics = Vec::new();
    for (path, file) in facts {
        for site in &file.worker_rate_limits {
            match &site.target {
                WorkerRateLimitTarget::LocalWorker(worker_index) => {
                    let worker = worker_index
                        .and_then(|idx| workers_by_index.get(&(path.as_path(), idx)).copied());
                    checks.push(finding(root, path, site.line, worker));
                }
                WorkerRateLimitTarget::ResolvedWorker {
                    file: origin,
                    worker_index,
                } => {
                    let worker = workers_by_index
                        .get(&(origin.as_path(), *worker_index))
                        .copied();
                    checks.push(finding(root, path, site.line, worker));
                }
                WorkerRateLimitTarget::Unresolved
                | WorkerRateLimitTarget::Imported { .. }
                | WorkerRateLimitTarget::Namespace { .. } => {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        file: relative_string(root, path),
                        line: site.line,
                        message: UNRESOLVED_MESSAGE.to_string(),
                    });
                }
            }
        }
    }
    (checks, diagnostics)
}

fn finding(root: &Path, path: &Path, line: usize, worker: Option<&InternalWorker>) -> CheckFinding {
    let queue_file = worker
        .and_then(|worker| worker.queue.as_ref())
        .map(|queue| relative_string(root, &queue.queue_file));
    let queue_name = worker.and_then(|worker| {
        worker
            .queue
            .as_ref()
            .map(|queue| queue.queue_name.clone())
            .or_else(|| worker.site.queue_name.clone())
    });
    let message = match &queue_name {
        Some(name) => format!(
            "`.rateLimit()` on the `{name}` worker applies per process only, so N worker processes run at N times the intended rate. Use the queue-level limiter or a provider-side wait."
        ),
        None => "`.rateLimit()` on a worker applies per process only, so N worker processes run at N times the intended rate. Use the queue-level limiter or a provider-side wait.".to_string(),
    };
    CheckFinding {
        kind: WORKER_RATE_LIMIT_KIND.to_string(),
        file: relative_string(root, path),
        line,
        queue_file,
        queue_name,
        job: None,
        message,
        severity: None,
    }
}

fn workers_by_file_index(workers: &[InternalWorker]) -> HashMap<(&Path, usize), &InternalWorker> {
    let mut counts: HashMap<&Path, usize> = HashMap::new();
    let mut out = HashMap::new();
    for worker in workers {
        let index = counts.entry(worker.site.file.as_path()).or_default();
        out.insert((worker.site.file.as_path(), *index), worker);
        *index += 1;
    }
    out
}
