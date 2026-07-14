use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

/// Immutable request-scoped ownership boundary for discovered files and source text.
///
/// Derived facts, graphs, and indexes live in the request contexts that consume this
/// dataset. The dataset itself never persists state beyond that request.
pub(crate) struct AnalysisDataset {
    root: PathBuf,
    observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
    visible_paths: Arc<super::ts_source::VisiblePathSnapshot>,
    root_sources: Arc<super::ts_source::SourceStore>,
    workspace: std::sync::OnceLock<Arc<super::workspaces::IndexedWorkspaceMap>>,
    config: OnceLock<Result<Arc<crate::config::v2::NoMistakesConfig>, Arc<str>>>,
    tsconfig: OnceLock<Result<Arc<super::ts_resolver::TsConfig>, Arc<str>>>,
    config_parses: AtomicUsize,
    tsconfig_parses: AtomicUsize,
}

impl AnalysisDataset {
    pub(crate) fn new(root: &Path) -> Self {
        Self::new_observed(root, None)
    }

    pub(crate) fn new_observed(
        root: &Path,
        observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
    ) -> Self {
        Self::from_snapshot_observed(
            root,
            Arc::new(super::ts_source::VisiblePathSnapshot::new_observed(
                root,
                observer.clone(),
            )),
            observer,
        )
    }

    pub(crate) fn from_snapshot(
        root: &Path,
        visible_paths: Arc<super::ts_source::VisiblePathSnapshot>,
    ) -> Self {
        Self::from_snapshot_observed(root, visible_paths, None)
    }

    pub(crate) fn from_snapshot_observed(
        root: &Path,
        visible_paths: Arc<super::ts_source::VisiblePathSnapshot>,
        observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
    ) -> Self {
        let root = super::ts_resolver::normalize_path(root);
        let root_sources = visible_paths.source_store_for(&root);
        Self {
            root,
            observer,
            visible_paths,
            root_sources,
            workspace: std::sync::OnceLock::new(),
            config: OnceLock::new(),
            tsconfig: OnceLock::new(),
            config_parses: AtomicUsize::new(0),
            tsconfig_parses: AtomicUsize::new(0),
        }
    }

    pub(crate) fn paths_for(&self, root: &Path) -> Arc<Vec<PathBuf>> {
        self.visible_paths.paths_for(root)
    }

    pub(crate) fn sources_for(&self, root: &Path) -> Arc<super::ts_source::SourceStore> {
        if super::ts_resolver::normalize_path(root) == self.root {
            return Arc::clone(&self.root_sources);
        }
        self.visible_paths.source_store_for(root)
    }
    pub(crate) fn config(
        &self,
        config_path: Option<&Path>,
    ) -> anyhow::Result<Arc<crate::config::v2::NoMistakesConfig>> {
        self.increment("manifest.requests", 1);
        if self.config.get().is_some() {
            self.increment("manifest.cache_hits", 1);
        }
        self.config
            .get_or_init(|| {
                self.config_parses.fetch_add(1, Ordering::Relaxed);
                self.increment("manifest.parses", 1);
                let result = crate::config::v2::load_v2_config_from_source_store(
                    &self.root,
                    config_path,
                    &self.paths_for(&self.root),
                    &self.root_sources,
                )
                .map(Arc::new)
                .map_err(|error| Arc::<str>::from(format!("{error:#}")));
                if result.is_err() {
                    self.increment("manifest.errors", 1);
                }
                result
            })
            .clone()
            .map_err(|error| anyhow::anyhow!(error.to_string()))
    }

    pub(crate) fn tsconfig(
        &self,
        tsconfig_path: Option<&Path>,
    ) -> anyhow::Result<Arc<super::ts_resolver::TsConfig>> {
        self.increment("manifest.requests", 1);
        if self.tsconfig.get().is_some() {
            self.increment("manifest.cache_hits", 1);
        }
        self.tsconfig
            .get_or_init(|| {
                self.tsconfig_parses.fetch_add(1, Ordering::Relaxed);
                self.increment("manifest.parses", 1);
                let result = super::ts_resolver::resolve_tsconfig_from_visible_and_sources(
                    tsconfig_path,
                    &self.root,
                    &self.paths_for(&self.root),
                    &self.root_sources,
                )
                .map(Arc::new)
                .map_err(|error| Arc::<str>::from(format!("{error:#}")));
                if result.is_err() {
                    self.increment("manifest.errors", 1);
                }
                result
            })
            .clone()
            .map_err(|error| anyhow::anyhow!(error.to_string()))
    }

    pub(crate) fn workspace(&self) -> Arc<super::workspaces::IndexedWorkspaceMap> {
        self.workspace
            .get_or_init(|| {
                super::workspaces::load_indexed_from_source_store(&self.root, &self.root_sources)
                    .map(Arc::new)
                    .unwrap_or_else(|_| Arc::new(super::workspaces::IndexedWorkspaceMap::default()))
            })
            .clone()
    }

    pub(crate) fn visible_paths(&self) -> &super::ts_source::VisiblePathSnapshot {
        self.visible_paths.as_ref()
    }

    pub(crate) fn visible_paths_arc(&self) -> Arc<super::ts_source::VisiblePathSnapshot> {
        Arc::clone(&self.visible_paths)
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    fn increment(&self, metric: &'static str, amount: u64) {
        if let Some(observer) = &self.observer {
            observer.increment(metric, amount);
        }
    }
}

#[cfg(test)]
mod tests;
