use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

mod job_options;
mod worker_rate_limit;
pub(crate) use job_options::{
    JobOptionEvidence, JobOptionFinding, JobOptionSite, JobOptionSpread, QueueDefaultOptions,
    RequiredJobOptions, RetentionEvidence, RetentionValue, UnboundedRetentionPolicy,
};
pub(crate) use worker_rate_limit::{
    ExportedWorkerBinding, WorkerRateLimitCall, WorkerRateLimitTarget,
};

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
    Unknown,
    Named(String),
    Local(u32),
    Flow(String),
    QueueName(String),
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

#[derive(Debug, Default, Clone)]
pub(crate) struct FileFacts {
    pub imports: Vec<ImportBinding>,
    pub namespace_import_sources: Vec<String>,
    pub enqueue_wrappers: Vec<super::enqueue_wrapper::WrapperSite>,
    pub stable_exported_strings: HashMap<String, String>,
    pub explicit_enqueue_exports: HashSet<String>,
    pub blocked_enqueue_exports: HashSet<String>,
    pub enqueue_export_aliases: HashMap<String, String>,
    pub exported_enqueue_properties: HashMap<String, HashMap<String, String>>,
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
    pub job_option_sites: Vec<JobOptionSite>,
    pub queue_defaults: Vec<QueueDefaultOptions>,
    pub job_option_findings: Vec<JobOptionFinding>,
    pub required_job_options: Option<RequiredJobOptions>,
    pub has_queue_package_import: bool,
    pub worker_rate_limits: Vec<WorkerRateLimitCall>,
    pub worker_bindings: HashMap<String, ExportedWorkerBinding>,
    pub worker_exports: HashMap<String, ExportedWorkerBinding>,
    pub worker_import_aliases: HashMap<String, String>,
    pub worker_namespace_exports: HashMap<String, (String, String)>,
    pub unbounded_job_retention: Option<UnboundedRetentionPolicy>,
    pub job_state_sites: Vec<super::job_state::JobStateSite>,
    pub job_state_as_record: Option<crate::config::v2::schema::JobStateAsRecordConfig>,
    pub worker_class_imports: HashMap<String, String>,
    pub worker_class_namespaces: HashMap<String, (String, String)>,
    pub worker_constructed_aliases: HashMap<String, String>,
    pub worker_constructed_namespaces: HashMap<String, (String, String)>,
}
