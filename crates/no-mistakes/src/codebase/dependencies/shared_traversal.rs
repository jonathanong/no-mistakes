pub(crate) struct SharedTraversalContext {
    session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    root: PathBuf,
    tsconfig: TsConfig,
    graph_files: graph::GraphFiles,
    dataset: std::sync::Arc<crate::codebase::analysis_dataset::AnalysisDataset>,
    config: crate::config::v2::NoMistakesConfig,
    config_path: Option<PathBuf>,
    prepared_graph: graph::PreparedGraphConfig,
    build_plan: graph::GraphBuildPlan,
    fact_plan: crate::codebase::ts_source::facts::TsFactPlan,
    fact_context: crate::codebase::ts_source::facts::TsFactContext,
    prepared_test_projects: Option<crate::codebase::test_discovery::PreparedTestProjects>,
    test_filter: crate::codebase::test_filter::TestFileFilter,
    facts: Option<crate::codebase::ts_source::facts::TsFactMap>,
    graph: Option<std::sync::Arc<graph::DepGraph>>,
    graph_cache: SharedBuildCache<EffectiveGraphPlanKey, graph::DepGraph>,
    symbol_index_cache: SharedBuildCache<GraphFileUniverseKey, graph::SymbolIndex>,
    import_resolution_cache: crate::codebase::ts_resolver::ImportResolutionCache,
    traversal_results: Vec<(TraversalCacheKey, Vec<graph::NodeEntry>)>,
    analysis_generation: u64,
    pub(crate) graph_builds: usize,
    pub(crate) symbol_index_builds: usize,
}

#[derive(PartialEq, Eq)]
struct TraversalCacheKey {
    generation: u64,
    dependents: bool,
    entrypoints: Vec<(PathBuf, NodeId, Option<String>)>,
    depth: Option<usize>,
    allowed: Vec<EdgeKind>,
    include_symbols: bool,
    import_only: bool,
}

impl SharedTraversalContext {
    pub(crate) fn session_arc(
        &self,
    ) -> std::sync::Arc<crate::codebase::analysis_session::AnalysisSession> {
        self.session.clone()
    }

    pub(crate) fn workspace(&self) -> &crate::codebase::workspaces::IndexedWorkspaceMap {
        self.prepared_graph.workspace()
    }
}

include!("shared_traversal/accessors.rs");

#[cfg(test)]
#[path = "shared_traversal/test_support.rs"]
mod test_support;
