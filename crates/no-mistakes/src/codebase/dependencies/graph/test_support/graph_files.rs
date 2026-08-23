use super::super::*;
use std::path::PathBuf;

impl GraphFiles {
    pub(crate) fn from_parts(
        mut all: Vec<PathBuf>,
        mut indexable: Vec<PathBuf>,
        visible: impl IntoIterator<Item = PathBuf>,
        mut resource_candidates: Vec<PathBuf>,
    ) -> Self {
        all.sort();
        all.dedup();
        indexable.sort();
        indexable.dedup();
        resource_candidates.sort();
        resource_candidates.dedup();
        let mut visible_paths: Vec<_> = visible.into_iter().collect();
        visible_paths.sort();
        visible_paths.dedup();
        let flags = all
            .iter()
            .map(|path| u8::from(visible_paths.binary_search(path).is_ok()))
            .collect();
        Self {
            all: std::sync::Arc::new(all),
            indexable: std::sync::Arc::new(indexable),
            visible: flags,
            canonical_visible: CanonicalVisible::empty(),
            resource_candidates: std::sync::Arc::new(resource_candidates),
        }
    }
}
