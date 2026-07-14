use super::PlaywrightFactPlan;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) struct PlaywrightModuleResolution {
    tsconfig: Arc<crate::codebase::ts_resolver::TsConfig>,
    workspace: Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
    visible_files: Arc<HashSet<PathBuf>>,
    cache: Arc<crate::codebase::ts_resolver::ImportResolutionCache>,
}

impl PlaywrightModuleResolution {
    pub(crate) fn new(
        tsconfig: Arc<crate::codebase::ts_resolver::TsConfig>,
        workspace: Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
        visible_files: Arc<HashSet<PathBuf>>,
    ) -> Self {
        Self {
            tsconfig,
            workspace,
            visible_files,
            cache: Arc::new(crate::codebase::ts_resolver::ImportResolutionCache::default()),
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
        let resolver = crate::codebase::ts_resolver::ImportResolver::new(&self.tsconfig)
            .with_visible(&self.visible_files)
            .with_shared_cache(&self.cache);
        let classification = resolver.classify_import(
            specifier,
            importing_file,
            &self.workspace,
            &self.visible_files,
        );
        if let Some(path) = classification.preferred_path() {
            return Some(ModuleIdentity::Path(
                crate::codebase::ts_resolver::normalize_path(path),
            ));
        }
        (classification.is_unresolved_external() && is_external_terminal(&resolver, specifier))
            .then(|| ModuleIdentity::External(specifier.to_string()))
    }
}

#[derive(PartialEq)]
enum ModuleIdentity {
    Path(PathBuf),
    External(String),
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

    pub(crate) fn set_module_resolution(&mut self, resolution: Arc<PlaywrightModuleResolution>) {
        self.module_resolution = Some(resolution);
    }

    pub(crate) fn module_resolution(&self) -> Option<&PlaywrightModuleResolution> {
        self.module_resolution.as_deref()
    }
}
