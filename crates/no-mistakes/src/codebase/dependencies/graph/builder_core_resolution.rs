enum GraphImportResolver<'a> {
    Scoped(crate::codebase::ts_resolver::ScopedImportResolver<'a>),
    Legacy(ImportResolver<'a>),
}

impl ImportResolution for GraphImportResolver<'_> {
    fn resolve(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf> {
        match self {
            Self::Scoped(resolver) => resolver.resolve(specifier, importing_file),
            Self::Legacy(resolver) => resolver.resolve(specifier, importing_file),
        }
    }

    fn visible_files(&self) -> Option<&HashSet<PathBuf>> {
        match self {
            Self::Scoped(resolver) => ImportResolution::visible_files(resolver),
            Self::Legacy(resolver) => ImportResolution::visible_files(resolver),
        }
    }

    fn classify_import(
        &self,
        specifier: &str,
        importing_file: &Path,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &HashSet<PathBuf>,
    ) -> crate::codebase::ts_resolver::ImportClassification {
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

fn graph_import_resolver<'a>(
    edge_inputs: &'a GraphEdgeBuildInputs<'a>,
    session: &'a crate::codebase::analysis_session::AnalysisSession,
) -> GraphImportResolver<'a> {
    match edge_inputs.tsconfig_catalog {
        Some(catalog) => {
            let resolver = crate::codebase::ts_resolver::ScopedImportResolver::new_in_session(
                catalog,
                edge_inputs.graph_files.visible(),
                session,
            );
            match edge_inputs.import_resolution_cache {
                Some(cache) => GraphImportResolver::Scoped(resolver.with_shared_cache(cache)),
                None => GraphImportResolver::Scoped(resolver),
            }
        }
        None => match edge_inputs.import_resolution_cache {
            Some(cache) => GraphImportResolver::Legacy(
                ImportResolver::new_observed(edge_inputs.tsconfig, session.observer().cloned())
                    .with_visible(edge_inputs.graph_files.visible())
                    .with_shared_cache(cache),
            ),
            None => GraphImportResolver::Legacy(ImportResolver::new_in_session(
                edge_inputs.tsconfig,
                Some(edge_inputs.graph_files.visible()),
                session,
            )),
        },
    }
}
