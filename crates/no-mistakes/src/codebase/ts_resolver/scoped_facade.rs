use super::{ImportClassification, ImportResolver, ScopedImportResolver};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Import resolution whose configuration is selected from the importing file.
/// This is the shared graph boundary during automatic workspace resolution.
pub(crate) trait ImportResolverFacade: Sync {
    fn resolve(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf>;

    fn visible_files(&self) -> Option<&HashSet<PathBuf>>;

    fn classify_import(
        &self,
        specifier: &str,
        importing_file: &Path,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &HashSet<PathBuf>,
    ) -> ImportClassification;
}

// Existing graph and runner-config call sites use this shorter name. New
// generic consumers use `ImportResolverFacade` to avoid colliding with their
// local `ImportResolution` context structs.
pub(crate) use ImportResolverFacade as ImportResolution;

impl<'a> ImportResolverFacade for ImportResolver<'a> {
    fn resolve(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf> {
        ImportResolver::resolve(self, specifier, importing_file)
    }

    fn visible_files(&self) -> Option<&HashSet<PathBuf>> {
        ImportResolver::visible_files(self)
    }

    fn classify_import(
        &self,
        specifier: &str,
        importing_file: &Path,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &HashSet<PathBuf>,
    ) -> ImportClassification {
        ImportResolver::classify_import(self, specifier, importing_file, workspace, visible_files)
    }
}

impl ImportResolverFacade for ScopedImportResolver<'_> {
    fn resolve(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf> {
        ScopedImportResolver::resolve(self, specifier, importing_file)
    }

    fn visible_files(&self) -> Option<&HashSet<PathBuf>> {
        self.visible.as_deref()
    }

    fn classify_import(
        &self,
        specifier: &str,
        importing_file: &Path,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &HashSet<PathBuf>,
    ) -> ImportClassification {
        ScopedImportResolver::classify_import(
            self,
            specifier,
            importing_file,
            workspace,
            visible_files,
        )
    }
}
