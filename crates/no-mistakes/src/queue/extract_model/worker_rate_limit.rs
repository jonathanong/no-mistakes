use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExportedWorkerBinding {
    Worker(Option<usize>),
    WorkerClass,
    NonWorker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorkerRateLimitTarget {
    LocalWorker(Option<usize>),
    ResolvedWorker {
        file: PathBuf,
        worker_index: Option<usize>,
    },
    Imported {
        local: String,
    },
    Namespace {
        source: String,
        exported: String,
    },
    ConstructedImported {
        local: String,
    },
    ConstructedNamespace {
        source: String,
        exported: String,
    },
    CallImported {
        local: String,
    },
    CallNamespace {
        source: String,
        exported: String,
    },
    Unresolved,
}

#[derive(Debug, Clone)]
pub(crate) struct WorkerRateLimitCall {
    pub line: usize,
    pub target: WorkerRateLimitTarget,
    pub mutated: bool,
}
