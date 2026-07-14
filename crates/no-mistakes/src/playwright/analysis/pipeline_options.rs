pub(crate) struct AnalysisOptions<'a> {
    pub(crate) require_routes: bool,
    pub(crate) skip_test_file_errors: bool,
    pub(crate) facts: Option<&'a dyn crate::codebase::dependencies::graph::TsFactLookup>,
    pub(crate) route_import_candidate: Option<(
        &'a crate::codebase::dependencies::graph::DepGraph,
        &'a crate::codebase::ts_resolver::TsConfig,
    )>,
    pub(crate) graph_file_universe: Option<&'a [std::path::PathBuf]>,
    pub(crate) occurrence_selection: super::pipeline_occurrences::CachedOccurrenceSelection,
    /// Canonical request-local candidate set. Every filesystem-facing phase
    /// of Playwright analysis must derive its inputs from this snapshot.
    pub(crate) snapshot: &'a crate::playwright::fsutil::VisiblePathSnapshot,
}
