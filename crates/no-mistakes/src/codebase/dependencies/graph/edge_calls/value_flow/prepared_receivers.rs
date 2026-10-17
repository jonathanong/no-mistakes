use super::*;
use crate::codebase::dependencies::extract::ValueReceiverMutations;

/// Histories stay fact-owned; graph preparation indexes their writer contexts once.
pub(super) struct PreparedReceiverMutations {
    histories: std::sync::Arc<ValueReceiverMutations>,
    pub(super) writer_contexts: FxHashSet<CallableId>,
    pub(super) module_writer: bool,
}
impl PreparedReceiverMutations {
    pub(super) fn new(histories: std::sync::Arc<ValueReceiverMutations>) -> Self {
        let mut writer_contexts = fx_set();
        let mut module_writer = false;
        let mut visited = fx_set();
        for history in histories.values().flat_map(|names| names.values()) {
            if !visited.insert(std::sync::Arc::as_ptr(history) as usize) {
                continue;
            }
            history.module_writer_contexts(&mut writer_contexts);
            if let Some(summary) = &history.summary {
                writer_contexts.extend(summary.contexts.iter().flatten().copied());
                module_writer |= summary.contexts.contains(&None);
            }
        }
        Self {
            histories,
            writer_contexts,
            module_writer,
        }
    }
}
impl std::ops::Deref for PreparedReceiverMutations {
    type Target = ValueReceiverMutations;
    fn deref(&self) -> &Self::Target {
        &self.histories
    }
}
