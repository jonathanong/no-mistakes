pub(crate) struct PreparedClientRelationships {
    graph_files: std::sync::Arc<crate::codebase::dependencies::graph::GraphFiles>,
    resolver: ImportResolver<'static>,
}

pub(super) struct ClientRelationshipInputs<'a> {
    pub(super) source_paths: &'a [PathBuf],
    pub(super) facts: &'a crate::codebase::ts_source::facts::TsFactMap,
    pub(super) prepared: &'a PreparedClientRelationships,
}

impl PreparedClientRelationships {
    fn new(
        source_files: &[PathBuf],
        tsconfig: &TsConfig,
        session: &crate::codebase::analysis_session::AnalysisSession,
    ) -> Self {
        let graph_files = std::sync::Arc::new(
            crate::codebase::dependencies::graph::GraphFiles::from_files(source_files.to_vec()),
        );
        let resolver = ImportResolver::new_owned_in_session(
            std::sync::Arc::new(tsconfig.clone()),
            std::sync::Arc::clone(&graph_files)
                as std::sync::Arc<dyn crate::codebase::ts_resolver::VisiblePathLookup>,
            session,
        );
        Self {
            graph_files,
            resolver,
        }
    }
}

fn client_source_paths(
    prepared: &PreparedServerAnalysis,
    filter: Option<&GlobSet>,
    test_filter: Option<&crate::codebase::test_filter::TestFileFilter>,
) -> Vec<PathBuf> {
    prepared
        .source_files
        .iter()
        .filter(|path| {
            let rel = path.strip_prefix(&prepared.root).unwrap_or(path);
            filter.is_none_or(|filter| filter.is_match(rel))
                && !test_filter.is_some_and(|filter| filter.is_match(&prepared.root, path))
        })
        .cloned()
        .collect()
}
