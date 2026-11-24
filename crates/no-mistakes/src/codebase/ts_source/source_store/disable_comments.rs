use super::{once_lock_slot, SourceStore};
use crate::codebase::ts_source::DisableCommentIndex;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

pub(super) type DisableCommentSlots =
    DashMap<PathBuf, Arc<OnceLock<Option<SharedDisableComments>>>>;
pub(crate) type SharedDisableComments = Arc<DisableCommentIndex<Arc<str>>>;

impl SourceStore {
    pub(crate) fn suppressions_for_path(&self, path: &Path) -> Option<SharedDisableComments> {
        let normalized = super::super::normalize_discovery_path(path);
        let slots = self.disable_comments.get_or_init(DashMap::new);
        let slot = once_lock_slot(slots, normalized);
        slot.get_or_init(|| {
            self.read_source_path(path).ok().map(|source| {
                Arc::new(DisableCommentIndex::new_observed(
                    source,
                    self.observer.clone(),
                ))
            })
        })
        .clone()
    }
}
