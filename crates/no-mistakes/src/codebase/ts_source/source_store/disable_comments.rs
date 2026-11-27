use super::{once_lock_slot, SourceStore};
use crate::codebase::ts_source::DisableCommentIndex;
use dashmap::DashMap;
use no_mistakes_rule_kernel::suppression::SuppressionSources;
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

impl SuppressionSources for SourceStore {
    type Index = SharedDisableComments;

    fn load_index(&self, root: &Path, candidate: &Path, is_absolute: bool) -> Option<Self::Index> {
        let path = if is_absolute {
            self.trusted_regular_path(candidate)
        } else {
            self.validated_regular_path(root, candidate)
        }?;
        self.suppressions_for_path(&path)
    }

    const ALLOW_ABSOLUTE_PATHS: bool = true;
}
