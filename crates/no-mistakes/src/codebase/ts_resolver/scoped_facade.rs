use super::{ImportClassification, ImportResolver, ResolverVisible, ScopedImportResolver};
use crate::codebase::ts_resolver::VisiblePathLookup;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Import resolution whose configuration is selected from the importing file.
/// This is the shared graph boundary during automatic workspace resolution.
pub(crate) trait ImportResolverFacade: Sync {
    fn resolve(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf>;

    /// Return conservative local resolution targets, including paths that no
    /// longer exist. This keeps deleted-file planning aligned with the
    /// importer-scoped resolver's aliases and compiler options.
    fn resolution_candidates(&self, specifier: &str, importing_file: &Path) -> BTreeSet<PathBuf>;

    fn visible_files(&self) -> Option<&dyn VisiblePathLookup>;

    fn classify_import(
        &self,
        specifier: &str,
        importing_file: &Path,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &dyn VisiblePathLookup,
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

    fn resolution_candidates(&self, specifier: &str, importing_file: &Path) -> BTreeSet<PathBuf> {
        ImportResolver::resolution_candidates(self, specifier, importing_file)
    }

    fn visible_files(&self) -> Option<&dyn VisiblePathLookup> {
        ImportResolver::visible_files(self)
    }

    fn classify_import(
        &self,
        specifier: &str,
        importing_file: &Path,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &dyn VisiblePathLookup,
    ) -> ImportClassification {
        ImportResolver::classify_import(self, specifier, importing_file, workspace, visible_files)
    }
}

impl ImportResolverFacade for ScopedImportResolver<'_> {
    fn resolve(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf> {
        ScopedImportResolver::resolve(self, specifier, importing_file)
    }

    fn resolution_candidates(&self, specifier: &str, importing_file: &Path) -> BTreeSet<PathBuf> {
        ScopedImportResolver::resolution_candidates(self, specifier, importing_file)
    }

    fn visible_files(&self) -> Option<&dyn VisiblePathLookup> {
        self.visible.as_ref().map(ResolverVisible::lookup)
    }

    fn classify_import(
        &self,
        specifier: &str,
        importing_file: &Path,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &dyn VisiblePathLookup,
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

/// Falls back to workspace package manifests for specifiers the wrapped
/// resolver leaves unresolved, so `@scope/package`, its `exports` subpaths, and
/// package `imports` (`#name`) reach the package source the same way import
/// edges reach it. Canonical call resolution wraps the graph's resolver in this
/// type so call edges, resolved call sites, and callable export resolutions
/// follow the same specifiers as the import graph.
///
/// The workspace map is consulted only when the wrapped resolver returns
/// `None`, and only against the visible-file lookup supplied at construction
/// (the graph's own files, exactly what import classification uses), so a
/// workspace entry outside the graph stays unresolved.
pub(crate) struct WorkspaceFallbackResolver<'a> {
    inner: &'a dyn ImportResolverFacade,
    workspace: &'a crate::codebase::workspaces::IndexedWorkspaceMap,
    visible: &'a dyn VisiblePathLookup,
}

impl<'a> WorkspaceFallbackResolver<'a> {
    pub(crate) fn new(
        inner: &'a dyn ImportResolverFacade,
        workspace: &'a crate::codebase::workspaces::IndexedWorkspaceMap,
        visible: &'a dyn VisiblePathLookup,
    ) -> Self {
        Self {
            inner,
            workspace,
            visible,
        }
    }
}

impl ImportResolverFacade for WorkspaceFallbackResolver<'_> {
    fn resolve(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf> {
        self.inner.resolve(specifier, importing_file).or_else(|| {
            self.workspace.resolve_specifier_from_file_visible(
                specifier,
                importing_file,
                self.visible,
            )
        })
    }

    fn resolution_candidates(&self, specifier: &str, importing_file: &Path) -> BTreeSet<PathBuf> {
        self.inner.resolution_candidates(specifier, importing_file)
    }

    fn visible_files(&self) -> Option<&dyn VisiblePathLookup> {
        self.inner.visible_files()
    }

    fn classify_import(
        &self,
        specifier: &str,
        importing_file: &Path,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &dyn VisiblePathLookup,
    ) -> ImportClassification {
        self.inner
            .classify_import(specifier, importing_file, workspace, visible_files)
    }
}
