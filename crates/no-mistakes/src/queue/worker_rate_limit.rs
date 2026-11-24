use crate::queue::extract::FileFacts;
use crate::queue::extract_helpers::is_queue_package;
use crate::queue::extract_model::{ExportedWorkerBinding, WorkerRateLimitTarget};
use crate::queue::graph_model::{CheckFinding, InternalWorker};
use crate::queue::source::relative_string;
use crate::queue::types::{Diagnostic, Severity};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[path = "worker_rate_limit_resolve.rs"]
mod resolve_exports;

pub(in crate::queue) use resolve_exports::resolve;

pub(crate) const WORKER_RATE_LIMIT_KIND: &str = "worker-rate-limit-per-process";

pub const WORKER_RATE_LIMIT_UNRESOLVED_MESSAGE: &str =
    "worker-rate-limit-per-process was not judged because the .rateLimit() receiver did not resolve";

pub fn is_worker_rate_limit_unresolved_diagnostic(diagnostic: &Diagnostic) -> bool {
    diagnostic.message == WORKER_RATE_LIMIT_UNRESOLVED_MESSAGE
}

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
                    let worker = worker_index
                        .and_then(|idx| workers_by_index.get(&(origin.as_path(), idx)).copied());
                    checks.push(finding(root, path, site.line, worker));
                }
                WorkerRateLimitTarget::Unresolved
                | WorkerRateLimitTarget::Imported { .. }
                | WorkerRateLimitTarget::Namespace { .. }
                | WorkerRateLimitTarget::ConstructedImported { .. }
                | WorkerRateLimitTarget::ConstructedNamespace { .. }
                | WorkerRateLimitTarget::CallImported { .. }
                | WorkerRateLimitTarget::CallNamespace { .. } => {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        file: relative_string(root, path),
                        line: site.line,
                        message: WORKER_RATE_LIMIT_UNRESOLVED_MESSAGE.to_string(),
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
        severity: super::graph_model::QueueFindingSeverity::Blocking,
        kind: WORKER_RATE_LIMIT_KIND.to_string(),
        file: relative_string(root, path),
        line,
        queue_file,
        queue_name,
        job: None,
        message,
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

fn is_queue_module(file: &FileFacts) -> bool {
    file.has_queue_package_import
        || !file.workers.is_empty()
        || !file.producers.is_empty()
        || !file.queue_bindings.is_empty()
        || !file.queue_exports.is_empty()
        || !file.enqueue_wrappers.is_empty()
        || file.star_exports.iter().any(|spec| is_queue_package(spec))
        || file.worker_exports.values().any(|binding| {
            matches!(
                binding,
                ExportedWorkerBinding::Worker(_) | ExportedWorkerBinding::WorkerClass
            )
        })
}
