use super::{EnqueuePriority, JobOptionSite, LocalCheck, ProducerBinding};

#[derive(Debug, Clone)]
pub(crate) struct WrapperSite {
    pub(crate) target: Target,
    pub options: JobOptionSite,
    pub checks: Vec<LocalCheck>,
    pub(crate) worker_index: Option<usize>,
    pub(crate) raw_job: String,
    pub(crate) priority: EnqueuePriority,
    pub(crate) dedup: bool,
    pub(crate) current_case: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Target {
    pub(crate) queue: Queue,
    pub(crate) job: Option<Name>,
}

#[derive(Debug, Clone)]
pub(crate) enum Queue {
    Unknown,
    Name(Name),
    Binding(ProducerBinding),
}

#[derive(Debug, Clone)]
pub(crate) enum Name {
    Literal(String),
    Import {
        local: String,
        property: Option<String>,
    },
}
