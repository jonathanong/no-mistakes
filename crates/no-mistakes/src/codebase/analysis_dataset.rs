use std::path::{Path, PathBuf};
use std::sync::Arc;

use manifest_cache::ManifestCache;

mod manifest_cache;

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
    config: ManifestCache<crate::config::v2::NoMistakesConfig>,
    tsconfig: ManifestCache<super::ts_resolver::TsConfig>,
}

impl AnalysisDataset {
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
            config: ManifestCache::default(),
            tsconfig: ManifestCache::default(),
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
        let visible_paths = self.paths_for(&self.root);
        let effective_path = crate::config::v2::effective_v2_config_path_from_visible(
            &self.root,
            config_path,
            &visible_paths,
        )
        .ok()
        .flatten();
        let key = effective_path
            .map(|path| super::ts_resolver::normalize_path(&path))
            .or_else(|| manifest_key(&self.root, config_path));
        let loaded = self.config.load(key, || {
            crate::config::v2::load_v2_config_from_source_store(
                &self.root,
                config_path,
                &visible_paths,
                &self.root_sources,
            )
            .map(Arc::new)
            .map_err(|error| Arc::<str>::from(format!("{error:#}")))
        });
        if loaded.loaded {
            self.increment("manifest.parses", 1);
            if loaded.value.is_err() {
                self.increment("manifest.errors", 1);
            }
        } else {
            self.increment("manifest.cache_hits", 1);
        }
        loaded
            .value
            .map_err(|error| anyhow::anyhow!(error.to_string()))
    }

    pub(crate) fn tsconfig(
        &self,
        tsconfig_path: Option<&Path>,
    ) -> anyhow::Result<Arc<super::ts_resolver::TsConfig>> {
        self.increment("manifest.requests", 1);
        let visible_paths = self.paths_for(&self.root);
        let key = tsconfig_path
            .and_then(|path| manifest_key(&self.root, Some(path)))
            .or_else(|| super::ts_resolver::find_tsconfig_from_visible(&self.root, &visible_paths));
        let loaded = self.tsconfig.load(key, || {
            super::ts_resolver::resolve_tsconfig_from_visible_and_sources(
                tsconfig_path,
                &self.root,
                &visible_paths,
                &self.root_sources,
            )
            .map(Arc::new)
            .map_err(|error| Arc::<str>::from(format!("{error:#}")))
        });
        if loaded.loaded {
            self.increment("manifest.parses", 1);
            if loaded.value.is_err() {
                self.increment("manifest.errors", 1);
            }
        } else {
            self.increment("manifest.cache_hits", 1);
        }
        loaded
            .value
            .map_err(|error| anyhow::anyhow!(error.to_string()))
    }

    pub(crate) fn workspace(&self) -> Arc<super::workspaces::IndexedWorkspaceMap> {
        self.workspace
            .get_or_init(|| {
                match super::workspaces::load_indexed_from_source_store(
                    &self.root,
                    &self.root_sources,
                ) {
                    Ok(workspace) => Arc::new(workspace),
                    Err(_) => Arc::new(super::workspaces::IndexedWorkspaceMap::default()),
                }
            })
            .clone()
    }

    pub(crate) fn visible_paths(&self) -> &super::ts_source::VisiblePathSnapshot {
        self.visible_paths.as_ref()
    }

    pub(crate) fn visible_paths_arc(&self) -> Arc<super::ts_source::VisiblePathSnapshot> {
        Arc::clone(&self.visible_paths)
    }

    fn increment(&self, metric: &'static str, amount: u64) {
        if let Some(observer) = &self.observer {
            observer.increment(metric, amount);
        }
    }
}

fn manifest_key(root: &Path, path: Option<&Path>) -> Option<PathBuf> {
    path.map(|path| {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        super::ts_resolver::normalize_path(&path)
    })
}

#[cfg(test)]
mod tests;
