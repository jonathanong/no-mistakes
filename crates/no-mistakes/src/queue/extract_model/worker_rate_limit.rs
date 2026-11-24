use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExportedWorkerBinding {
    Worker(usize),
    NonWorker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorkerRateLimitTarget {
    LocalWorker(Option<usize>),
    ResolvedWorker { file: PathBuf, worker_index: usize },
    Imported { local: String },
    Namespace { source: String, exported: String },
    Unresolved,
}

#[derive(Debug, Clone)]
pub(crate) struct WorkerRateLimitCall {
    pub line: usize,
    pub target: WorkerRateLimitTarget,
}
