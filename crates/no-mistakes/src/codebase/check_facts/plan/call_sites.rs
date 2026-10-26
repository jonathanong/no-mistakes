use super::CheckFactPlan;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

impl CheckFactPlan {
    /// Add exact call-source demand without widening generic graph consumers.
    #[doc(hidden)]
    pub fn require_call_site_paths(&mut self, paths: &[PathBuf]) {
        if paths.iter().all(|path| self.call_site_paths.contains(path)) {
            return;
        }
        Arc::make_mut(&mut self.call_site_paths).extend(
            paths
                .iter()
                .map(|path| crate::codebase::ts_resolver::normalize_path(path)),
        );
    }

    pub(crate) fn requires_call_sites(&self, path: &Path) -> bool {
        self.graph.call_sites
            || (!self.call_site_paths.is_empty()
                && (self.call_site_paths.contains(path)
                    || self
                        .call_site_paths
                        .contains(&crate::codebase::ts_resolver::normalize_path(path))))
    }

    pub(super) fn merge_call_site_paths(&mut self, other: &Self) {
        if other.call_site_paths.is_empty()
            || Arc::ptr_eq(&self.call_site_paths, &other.call_site_paths)
        {
            return;
        }
        if self.call_site_paths.is_empty() {
            self.call_site_paths = Arc::clone(&other.call_site_paths);
        } else {
            Arc::make_mut(&mut self.call_site_paths).extend(other.call_site_paths.iter().cloned());
        }
    }
}

#[cfg(test)]
mod tests;
