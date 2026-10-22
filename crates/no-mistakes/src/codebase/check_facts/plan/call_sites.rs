use super::CheckFactPlan;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl CheckFactPlan {
    /// Keep path-scoped call facts when collecting a graph-only source.
    pub(crate) fn graph_only_projection(&self) -> Self {
        Self {
            graph: self.graph,
            call_site_paths: Arc::clone(&self.call_site_paths),
            graph_context: self.graph_context.clone(),
            integration_runner_configs: self.integration_runner_configs.clone(),
            ..Self::default()
        }
    }

    pub(crate) fn includes_call_site_path(&self, path: &Path) -> bool {
        self.call_site_paths
            .binary_search_by(|candidate| candidate.as_path().cmp(path))
            .is_ok()
    }

    pub(crate) fn add_call_site_paths(&mut self, paths: &[PathBuf]) {
        if paths.iter().all(|path| self.includes_call_site_path(path)) {
            return;
        }
        let selected = Arc::make_mut(&mut self.call_site_paths);
        selected.extend(paths.iter().cloned());
        selected.sort();
        selected.dedup();
    }

    pub(super) fn include_call_site_plan(&mut self, other: &Self) {
        if other.call_site_paths.is_empty() {
            return;
        }
        if self.call_site_paths.is_empty() {
            self.call_site_paths = Arc::clone(&other.call_site_paths);
        } else if !Arc::ptr_eq(&self.call_site_paths, &other.call_site_paths) {
            self.add_call_site_paths(&other.call_site_paths);
        }
    }
}
