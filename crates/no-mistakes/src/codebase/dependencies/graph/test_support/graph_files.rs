use super::super::*;
use std::path::PathBuf;

impl GraphFiles {
    pub(crate) fn from_parts(
        mut all: Vec<PathBuf>,
        mut indexable: Vec<PathBuf>,
        visible: impl IntoIterator<Item = PathBuf>,
        mut resource_candidates: Vec<PathBuf>,
    ) -> Self {
        crate::codebase::ts_source::sort_os_str_paths(&mut all);
        all.dedup();
        crate::codebase::ts_source::sort_os_str_paths(&mut indexable);
        indexable.dedup();
        crate::codebase::ts_source::sort_os_str_paths(&mut resource_candidates);
        resource_candidates.dedup();
        let mut visible_paths: Vec<_> = visible.into_iter().collect();
        crate::codebase::ts_source::sort_os_str_paths(&mut visible_paths);
        visible_paths.dedup();
        let flags = all
            .iter()
            .map(|path| {
                u8::from(
                    visible_paths
                        .binary_search_by(|candidate| {
                            crate::codebase::ts_source::cmp_os_str_paths(candidate, path)
                        })
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
