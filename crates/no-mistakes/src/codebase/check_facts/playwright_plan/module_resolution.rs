use super::PlaywrightFactPlan;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[path = "module_resolution_catalog.rs"]
mod catalog;
use catalog::CatalogModuleResolver;

#[cfg(test)]
#[path = "module_resolution_test_support.rs"]
mod tests;

pub(crate) struct PlaywrightModuleResolution {
    tsconfig: PlaywrightTsConfig,
    workspace: Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
    visible_files: Arc<HashSet<PathBuf>>,
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
        visible_files: Arc<HashSet<PathBuf>>,
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
        visible_files: Arc<HashSet<PathBuf>>,
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
            (Some(configured), Some(imported)) => configured == imported,
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
                    return Some(ModuleIdentity::Path(self.remapper.remap(path)));
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

#[derive(PartialEq)]
enum ModuleIdentity {
    Path(PathBuf),
    External(String),
}

fn identity_from_resolver(
    resolver: &crate::codebase::ts_resolver::ImportResolver<'_>,
    specifier: &str,
    importing_file: &Path,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &HashSet<PathBuf>,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> Option<ModuleIdentity> {
    let classification =
        resolver.classify_import(specifier, importing_file, workspace, visible_files);
    if let Some(path) = classification.preferred_path() {
        return Some(ModuleIdentity::Path(remapper.remap(path)));
    }
    (classification.is_unresolved_external() && is_external_terminal(resolver, specifier))
        .then(|| ModuleIdentity::External(specifier.to_string()))
}

fn is_external_terminal(
    resolver: &crate::codebase::ts_resolver::ImportResolver<'_>,
    specifier: &str,
) -> bool {
    !specifier.starts_with('.')
        && !specifier.starts_with('/')
        && !specifier.starts_with('#')
        && !resolver.matches_alias(specifier)
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
