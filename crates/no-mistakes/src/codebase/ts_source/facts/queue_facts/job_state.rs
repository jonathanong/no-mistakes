use super::ProducerBinding;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum JobStatePattern {
    GetJobStateGuard,
    ListingFeedsAction,
}

#[derive(Debug, Clone)]
pub(crate) struct JobStateSite {
    pub line: usize,
    pub binding: ProducerBinding,
    /// When the guarded/fed action is `.add` / `.addBulk`, its receiver must also
    /// resolve to a configured queue (so `Set.add` does not trigger a finding).
    pub action_binding: Option<ProducerBinding>,
    pub queue_file: Option<PathBuf>,
    pub queue_name: Option<String>,
    pub unresolved_receiver: bool,
    pub uncertain_item: bool,
    pub pattern: JobStatePattern,
}
