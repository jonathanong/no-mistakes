use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::dependencies::graph::GraphFiles;
use crate::codebase::ts_resolver::{ScopedImportResolver, TsConfigCatalog};
use crate::codebase::workspace_resolution::IndexedWorkspaceMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(super) trait FragmentResolver: Sync {
    fn resolve(&self, specifier: &str, importer: &Path, explicit: bool) -> Option<Option<PathBuf>>;
}

pub(crate) struct FragmentImportResolution<'a> {
    session: &'a AnalysisSession,
    catalog: &'a TsConfigCatalog,
    workspace: &'a IndexedWorkspaceMap,
    files: GraphFiles,
    has_aliases: bool,
    resolver: OnceLock<ScopedImportResolver<'a>>,
}

impl<'a> FragmentImportResolution<'a> {
    pub(crate) fn new(
        session: &'a AnalysisSession,
        catalog: &'a TsConfigCatalog,
        workspace: &'a IndexedWorkspaceMap,
        files: &[PathBuf],
    ) -> Self {
        Self {
            session,
            catalog,
            workspace,
            files: GraphFiles::from_files(files.to_vec()),
            has_aliases: catalog.has_path_aliases(),
            resolver: OnceLock::new(),
        }
    }
}

impl FragmentResolver for FragmentImportResolution<'_> {
    fn resolve(&self, specifier: &str, importer: &Path, explicit: bool) -> Option<Option<PathBuf>> {
        let workspace = self
            .workspace
            .recognizes_specifier_from(specifier, importer);
        if !explicit && !self.has_aliases && !workspace {
            return None;
        }
        let resolver = self.resolver.get_or_init(|| {
            ScopedImportResolver::new_in_session(self.catalog, &self.files, self.session)
        });
        let alias = self.has_aliases && resolver.matches_paths_alias(specifier, importer);
        if !explicit && !alias && !workspace {
            return None;
        }
        self.session
            .record_work("postgres_sql_fragment_escape.module_candidates", 1);
        let classification =
            resolver.classify_import(specifier, importer, self.workspace, &self.files);
        if classification.is_unresolved_external()
            && !alias
            && !specifier.starts_with('.')
            && !Path::new(specifier).is_absolute()
        {
            return None;
        }
        let resolved = classification
            .resolver_path()
            .and_then(|target| self.files.visible_path(target))
            .or_else(|| {
                (!alias)
                    .then(|| self.files.visible_path(classification.workspace_path()?))
                    .flatten()
            });
        Some(resolved.map(Path::to_path_buf))
    }
}
