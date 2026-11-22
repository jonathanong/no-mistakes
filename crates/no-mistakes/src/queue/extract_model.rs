use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct ImportBinding {
    pub local: String,
    pub imported: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ProducerSite {
    pub file: PathBuf,
    pub line: usize,
    pub binding: ProducerBinding,
    pub job: Option<String>,
    /// Local name of an imported string constant used as the job name.
    pub job_import: Option<String>,
    pub raw_job: Option<String>,
    /// Index of the factory or `new Worker` processor that contains this enqueue.
    pub worker_index: Option<usize>,
    /// The add reuses the active job's dedup id or job id. Judged after queue resolution.
    pub self_dedup: bool,
    /// Static `priority` on this enqueue. `Dynamic` makes the queue abstain.
    pub priority: EnqueuePriority,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ProducerBinding {
    Named(String),
    Local(u32),
    Flow(String),
}

#[derive(Debug, Clone)]
pub(crate) enum LocalQueue {
    Known(String),
    ImportedName(String),
    ImportedQueue(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum EnqueuePriority {
    Unprioritized,
    Prioritized(f64),
    Dynamic,
}

#[derive(Debug, Clone)]
pub(crate) struct WorkerSite {
    pub file: PathBuf,
    pub line: usize,
    pub queue_name: Option<String>,
    /// Local identifier of a queue-name argument that is imported, not a literal.
    pub queue_name_import: Option<String>,
    pub jobs: Vec<String>,
    pub processor_specifier: Option<String>,
    pub processor_file: Option<PathBuf>,
    pub wildcard: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum JobIdQueue {
    Named(ProducerBinding),
    Namespace { source: String, exported: String },
}

#[derive(Debug, Clone)]
pub(crate) struct LocalCheck {
    pub line: usize,
    pub kind: String,
    pub job: Option<String>,
    pub message: String,
}

/// A `RateLimitError` throw or `.rateLimit()` call inside a processor.
#[derive(Debug, Clone)]
pub(crate) struct RateLimitSite {
    pub line: usize,
    pub worker_index: usize,
}

/// Keys present on a job-options object, plus spreads resolved at the end of the file.
#[derive(Debug, Clone, Default)]
pub(crate) struct JobOptionEvidence {
    pub keys: BTreeSet<String>,
    pub open: bool,
    pub spreads: Vec<String>,
    /// When set, finish folds each name in `spreads` as this member, in order.
    pub member: Option<String>,
}

impl JobOptionEvidence {
    pub(crate) fn open() -> Self {
        Self {
            open: true,
            ..Self::default()
        }
    }

    pub(crate) fn absorb(&mut self, other: &Self) {
        self.keys.extend(other.keys.iter().cloned());
        self.spreads.extend(other.spreads.iter().cloned());
        self.open |= other.open;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct JobOptionSite {
    pub line: usize,
    pub binding: Option<String>,
    pub queue_name: Option<String>,
    pub queue_file: Option<PathBuf>,
    pub job: Option<String>,
    pub evidence: JobOptionEvidence,
}

#[derive(Debug, Clone)]
pub(crate) struct QueueDefaultOptions {
    pub binding: Option<String>,
    pub queue_name: Option<String>,
    /// Const name whose object is the whole queue-options argument.
    pub container: Option<String>,
    pub evidence: JobOptionEvidence,
}

#[derive(Debug, Clone)]
pub(crate) struct JobOptionFinding {
    pub line: usize,
    pub queue_file: Option<PathBuf>,
    pub queue_name: Option<String>,
    pub job: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RequiredJobOptions {
    pub global: Vec<String>,
    pub by_queue: BTreeMap<String, Vec<String>>,
}

impl RequiredJobOptions {
    pub(crate) fn for_queue(&self, name: Option<&str>) -> &[String] {
        name.and_then(|name| self.by_queue.get(name))
            .map(Vec::as_slice)
            .unwrap_or(&self.global)
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct FileFacts {
    pub imports: Vec<ImportBinding>,
    pub queue_bindings: HashMap<String, String>,
    pub queue_declarations: HashMap<u32, LocalQueue>,
    pub queue_exports: HashMap<String, String>,
    pub producers: Vec<ProducerSite>,
    pub workers: Vec<WorkerSite>,
    pub diagnostics: Vec<(usize, String)>,
    /// `export const NAME = "value"` strings other files can import.
    pub exported_strings: HashMap<String, String>,
    /// Named imports that may be a string constant or a queue binding.
    pub string_imports: HashMap<String, (String, String)>,
    /// `export { imported as exported } from`. Kept off `string_imports` so a
    /// same-named local import is not overwritten.
    pub export_from: HashMap<String, (String, String)>,
    /// `RateLimitError` throws and `.rateLimit()` calls inside a processor.
    pub rate_limits: Vec<RateLimitSite>,
    /// Local queue binding whose name argument is an imported identifier.
    pub pending_queue_bindings: HashMap<String, String>,
    /// `(exported name, local binding)` aliases of a queue object.
    pub reexports: Vec<(String, String)>,
    /// `export * from` specifiers. Lookup copies queue exports through these.
    pub star_exports: Vec<String>,
    /// Static `job.name` / `case` names in this file, used when it is an imported processor.
    pub referenced_jobs: Vec<String>,
    /// A job-name use outside a literal comparison or switch label keeps imported processors open.
    pub referenced_jobs_dynamic: bool,
    pub checks: Vec<LocalCheck>,
    pub job_id_sites: Vec<(JobIdQueue, LocalCheck)>,
    /// Same-file `const` integers seen while extracting this file.
    pub const_numbers: HashMap<String, f64>,
    /// Bindings whose bulk enqueue list was not a closed list of object literals.
    pub opaque_priority_bindings: HashSet<ProducerBinding>,
    /// Resolved `(queue file, queue name)` pairs filled after binding resolution.
    pub opaque_priority_queues: Vec<(PathBuf, String)>,
    pub const_objects: HashMap<String, JobOptionEvidence>,
    pub const_queue_defaults: HashMap<String, JobOptionEvidence>,
    pub const_opts: HashMap<String, JobOptionEvidence>,
    pub job_option_sites: Vec<JobOptionSite>,
    pub queue_defaults: Vec<QueueDefaultOptions>,
    pub job_option_findings: Vec<JobOptionFinding>,
    pub required_job_options: Option<RequiredJobOptions>,
}
