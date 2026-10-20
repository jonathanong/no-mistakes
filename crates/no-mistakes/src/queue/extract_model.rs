use std::collections::HashMap;
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
    pub binding: String,
    pub job: Option<String>,
    /// Local name of an imported string constant used as the job name.
    pub job_import: Option<String>,
    pub raw_job: Option<String>,
    /// Index of the factory or `new Worker` processor that contains this enqueue.
    pub worker_index: Option<usize>,
    /// The add reuses the active job's dedup id or job id. Judged after queue resolution.
    pub self_dedup: bool,
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
pub(crate) struct LocalCheck {
    pub line: usize,
    pub kind: String,
    pub job: Option<String>,
    pub message: String,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct FileFacts {
    pub imports: Vec<ImportBinding>,
    pub queue_bindings: HashMap<String, String>,
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
}
