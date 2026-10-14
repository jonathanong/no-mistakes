use super::*;
use crate::codebase::dependencies::extract::ValueReceiverMutations;

/// Histories stay fact-owned; graph preparation indexes their writer contexts once.
pub(super) struct PreparedReceiverMutations {
    histories: std::sync::Arc<ValueReceiverMutations>,
    pub(super) writer_contexts: FxHashSet<CallableId>,
}
impl PreparedReceiverMutations {
    pub(super) fn new(histories: std::sync::Arc<ValueReceiverMutations>) -> Self {
        let mut writer_contexts = fx_set();
        let mut visited = fx_set();
        for history in histories.values().flat_map(|names| names.values()) {
            if !visited.insert(std::sync::Arc::as_ptr(history) as usize) {
                continue;
            }
            if let Some(summary) = &history.summary {
                writer_contexts.extend(summary.contexts.iter().flatten().copied());
            }
        }
        Self {
            histories,
            writer_contexts,
        }
    }
}
impl std::ops::Deref for PreparedReceiverMutations {
    type Target = ValueReceiverMutations;
    fn deref(&self) -> &Self::Target {
        &self.histories
    }
}
