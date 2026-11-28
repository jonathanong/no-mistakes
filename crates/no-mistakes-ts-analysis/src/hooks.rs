//! Inverted runtime hooks for `no-mistakes-ts-analysis`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use crate::workspace_resolution::WorkspaceMap;

pub trait MetricsObserver: Send + Sync {
    fn increment(&self, metric: &'static str, amount: u64);
}

pub trait AnalysisHooks: Send + Sync {
    fn current_observer(&self) -> Option<Arc<dyn MetricsObserver>>;
    fn discover_visible_paths(&self, root: &Path) -> Vec<PathBuf>;
    fn load_workspace_from_files(
        &self,
        root: &Path,
        visible_paths: &[PathBuf],
    ) -> Option<WorkspaceMap>;
}

static REGISTERED_HOOKS: OnceLock<&'static dyn AnalysisHooks> = OnceLock::new();

pub fn register_hooks(hooks: &'static dyn AnalysisHooks) {
    let _ = REGISTERED_HOOKS.set(hooks);
}

pub fn current_observer() -> Option<Arc<dyn MetricsObserver>> {
    current_observer_with(REGISTERED_HOOKS.get().copied())
}

pub fn discover_visible_paths(root: &Path) -> Vec<PathBuf> {
    discover_visible_paths_with(REGISTERED_HOOKS.get().copied(), root)
}

pub fn load_workspace_from_files(root: &Path, visible_paths: &[PathBuf]) -> Option<WorkspaceMap> {
    load_workspace_from_files_with(REGISTERED_HOOKS.get().copied(), root, visible_paths)
}

fn current_observer_with(hooks: Option<&dyn AnalysisHooks>) -> Option<Arc<dyn MetricsObserver>> {
    hooks.and_then(AnalysisHooks::current_observer)
}

fn discover_visible_paths_with(hooks: Option<&dyn AnalysisHooks>, root: &Path) -> Vec<PathBuf> {
    match hooks {
        Some(hooks) => hooks.discover_visible_paths(root),
        None => Vec::new(),
    }
}

fn load_workspace_from_files_with(
    hooks: Option<&dyn AnalysisHooks>,
    root: &Path,
    visible_paths: &[PathBuf],
) -> Option<WorkspaceMap> {
    hooks.and_then(|hooks| hooks.load_workspace_from_files(root, visible_paths))
}

#[cfg(test)]
mod tests;
