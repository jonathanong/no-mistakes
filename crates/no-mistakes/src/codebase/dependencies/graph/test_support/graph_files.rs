use super::super::*;
use std::path::PathBuf;

impl GraphFiles {
    pub(crate) fn from_parts(
        mut all: Vec<PathBuf>,
        mut indexable: Vec<PathBuf>,
        visible: impl IntoIterator<Item = PathBuf>,
        mut resource_candidates: Vec<PathBuf>,
    ) -> Self {
        all.sort_by(|left, right| left.as_os_str().cmp(right.as_os_str()));
        all.dedup();
        indexable.sort_by(|left, right| left.as_os_str().cmp(right.as_os_str()));
        indexable.dedup();
        resource_candidates.sort_by(|left, right| left.as_os_str().cmp(right.as_os_str()));
        resource_candidates.dedup();
        let mut visible_paths: Vec<_> = visible.into_iter().collect();
        visible_paths.sort_by(|left, right| left.as_os_str().cmp(right.as_os_str()));
        visible_paths.dedup();
        let flags = all
            .iter()
            .map(|path| {
                u8::from(
                    visible_paths
                        .binary_search_by(|candidate| candidate.as_os_str().cmp(path.as_os_str()))
                        .is_ok(),
                )
            })
            .collect();
        Self {
            all: std::sync::Arc::new(all),
            indexable: std::sync::Arc::new(indexable),
            visible: flags,
            canonical_visible: CanonicalVisible::empty(),
            scoped_visible: std::sync::OnceLock::new(),
            resource_candidates: std::sync::Arc::new(resource_candidates),
        }
    }
}
