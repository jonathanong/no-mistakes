pub(crate) enum ProjectImportResolver<'a> {
    Scoped(Box<ScopedImportResolver<'a>>),
    Legacy(ImportResolver<'a>),
}

impl<'a> ProjectImportResolver<'a> {
    pub(crate) fn new(
        tsconfig: &'a TsConfig,
        catalog: Option<&'a TsConfigCatalog>,
        visible: &'a dyn VisiblePathLookup,
        shared_cache: Option<&'a ImportResolutionCache>,
        session: &'a crate::codebase::analysis_session::AnalysisSession,
    ) -> Self {
        match catalog {
            Some(catalog) => {
                let resolver = ScopedImportResolver::new_in_session(catalog, visible, session);
                Self::Scoped(Box::new(match shared_cache {
                    Some(cache) => resolver.with_shared_cache(cache),
                    None => resolver,
                }))
            }
            None => {
                let resolver = ImportResolver::new_in_session(tsconfig, Some(visible), session);
                Self::Legacy(match shared_cache {
                    Some(cache) => resolver.with_shared_cache(cache),
                    None => resolver,
                })
            }
        }
    }
}

impl ImportResolution for ProjectImportResolver<'_> {
    fn resolve(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf> {
        match self {
            Self::Scoped(resolver) => resolver.resolve(specifier, importing_file),
            Self::Legacy(resolver) => resolver.resolve(specifier, importing_file),
        }
    }

    fn resolution_candidates(
        &self,
        specifier: &str,
        importing_file: &Path,
    ) -> std::collections::BTreeSet<PathBuf> {
        match self {
            Self::Scoped(resolver) => resolver.resolution_candidates(specifier, importing_file),
            Self::Legacy(resolver) => resolver.resolution_candidates(specifier, importing_file),
        }
    }

    fn visible_files(&self) -> Option<&dyn VisiblePathLookup> {
        match self {
            Self::Scoped(resolver) => ImportResolution::visible_files(resolver.as_ref()),
            Self::Legacy(resolver) => ImportResolution::visible_files(resolver),
        }
    }

    fn classify_import(
        &self,
        specifier: &str,
        importing_file: &Path,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &dyn VisiblePathLookup,
    ) -> ImportClassification {
        match self {
            Self::Scoped(resolver) => {
                resolver.classify_import(specifier, importing_file, workspace, visible_files)
            }
            Self::Legacy(resolver) => {
                resolver.classify_import(specifier, importing_file, workspace, visible_files)
            }
        }
    }
}
