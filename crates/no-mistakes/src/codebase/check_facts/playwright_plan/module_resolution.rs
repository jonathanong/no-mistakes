use super::PlaywrightFactPlan;
use crate::fx::PathSet;
use std::path::Path;
use std::sync::Arc;

#[path = "module_resolution_catalog.rs"]
mod catalog;
use catalog::CatalogModuleResolver;
#[path = "module_resolution_path.rs"]
mod path_match;
use path_match::{
    identity_from_resolver, looks_like_repo_relative_module, path_ends_with_module, ModuleIdentity,
};

#[cfg(test)]
#[path = "module_resolution_test_support.rs"]
mod tests;

pub(crate) struct PlaywrightModuleResolution {
    tsconfig: PlaywrightTsConfig,
    workspace: Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
    visible_files: Arc<PathSet>,
    remapper: Arc<crate::codebase::ts_source::FrozenPathRemapper>,
    cache: Arc<crate::codebase::ts_resolver::ImportResolutionCache>,
    catalog_resolver: Option<CatalogModuleResolver>,
}

enum PlaywrightTsConfig {
    Single(Arc<crate::codebase::ts_resolver::TsConfig>),
    Catalog,
}

impl PlaywrightModuleResolution {
    pub(crate) fn new(
        tsconfig: Arc<crate::codebase::ts_resolver::TsConfig>,
        workspace: Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
        visible_files: Arc<PathSet>,
    ) -> Self {
        let remapper = Arc::new(crate::codebase::ts_source::FrozenPathRemapper::from_paths(
            visible_files.iter().cloned(),
        ));
        Self {
            tsconfig: PlaywrightTsConfig::Single(tsconfig),
            workspace,
            visible_files: Arc::clone(&visible_files),
            remapper,
            cache: Arc::new(crate::codebase::ts_resolver::ImportResolutionCache::default()),
            catalog_resolver: None,
        }
    }

    pub(crate) fn with_catalog(
        tsconfig_catalog: Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
        workspace: Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
        visible_files: Arc<PathSet>,
    ) -> Self {
        let remapper = Arc::new(crate::codebase::ts_source::FrozenPathRemapper::from_paths(
            visible_files.iter().cloned(),
        ));
        let catalog_resolver = CatalogModuleResolver::new(tsconfig_catalog, Arc::clone(&remapper));
        Self {
            tsconfig: PlaywrightTsConfig::Catalog,
            workspace,
            visible_files: Arc::clone(&visible_files),
            remapper,
            cache: Arc::new(crate::codebase::ts_resolver::ImportResolutionCache::default()),
            catalog_resolver: Some(catalog_resolver),
        }
    }

    pub(crate) fn modules_match(
        &self,
        configured: &str,
        imported: &str,
        importing_file: &Path,
    ) -> bool {
        match (
            self.identity(configured, importing_file),
            self.identity(imported, importing_file),
        ) {
            (Some(configured_id), Some(imported_id)) => configured_id == imported_id,
            (_, Some(ModuleIdentity::Path(imported_path)))
                if looks_like_repo_relative_module(configured) =>
            {
                path_ends_with_module(&imported_path, configured)
            }
            _ => false,
        }
    }

    fn identity(&self, specifier: &str, importing_file: &Path) -> Option<ModuleIdentity> {
        match &self.tsconfig {
            PlaywrightTsConfig::Single(tsconfig) => {
                let resolver = crate::codebase::ts_resolver::ImportResolver::new(tsconfig)
                    .with_visible(&self.visible_files)
                    .with_shared_cache(&self.cache);
                identity_from_resolver(
                    &resolver,
                    specifier,
                    importing_file,
                    &self.workspace,
                    &self.visible_files,
                    &self.remapper,
                )
            }
            PlaywrightTsConfig::Catalog => {
                let resolver = self.catalog_resolver.as_ref().expect("catalog facade");
                let classification = resolver.classify(specifier, importing_file, &self.workspace);
                if let Some(path) = classification.import_classification.preferred_path() {
                    return self.remapper.remap(path).map(ModuleIdentity::Path);
                }
                (classification
                    .import_classification
                    .is_unresolved_external()
                    && classification.is_external_terminal)
                    .then(|| ModuleIdentity::External(specifier.to_string()))
            }
        }
    }
}

impl PlaywrightFactPlan {
    pub(crate) fn configure_module_resolution(
        &mut self,
        tsconfig: Arc<crate::codebase::ts_resolver::TsConfig>,
        workspace: Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
        visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
        root: &Path,
    ) {
        let visible_files = Arc::new(
            visible_paths
                .paths_for(root)
                .iter()
                .map(|path| crate::codebase::ts_resolver::normalize_path(path))
                .collect(),
        );
        self.set_module_resolution(Arc::new(PlaywrightModuleResolution::new(
            tsconfig,
            workspace,
            visible_files,
        )));
    }

    pub(crate) fn configure_module_resolution_with_catalog(
        &mut self,
        tsconfig_catalog: Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
        workspace: Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
        visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
        root: &Path,
    ) {
        let visible_files = Arc::new(
            visible_paths
                .paths_for(root)
                .iter()
                .map(|path| crate::codebase::ts_resolver::normalize_path(path))
                .collect(),
        );
        self.set_module_resolution(Arc::new(PlaywrightModuleResolution::with_catalog(
            tsconfig_catalog,
            workspace,
            visible_files,
        )));
    }

    pub(crate) fn set_module_resolution(&mut self, resolution: Arc<PlaywrightModuleResolution>) {
        self.module_resolution = Some(resolution);
    }

    pub(crate) fn module_resolution(&self) -> Option<&PlaywrightModuleResolution> {
        self.module_resolution.as_deref()
    }
}
