#[derive(Debug, Clone, PartialEq)]
pub struct EnqueueCall {
    /// Local binding identifier used (e.g. `emailsQueue`).
    pub binding: String,
    /// Job name literal if present.
    pub job: Option<String>,
    pub line: u32,
}

/// A `new Worker('queueName', handler)` declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkerDeclaration {
    /// Queue name string literal (e.g. `"emails"`), if present.
    pub queue_name: Option<String>,
    /// Import specifier of the processors namespace (`import * as processors from '...'`).
    pub processors_specifier: Option<String>,
    pub line: u32,
}

/// All queue-related usage patterns extracted from a source file.
#[derive(Debug, Default, Clone)]
pub struct QueueUsage {
    /// `(local_binding, import_specifier)` pairs for named imports.
    pub imports: Vec<(String, String)>,
    pub enqueue_calls: Vec<EnqueueCall>,
    pub worker_declarations: Vec<WorkerDeclaration>,
}
