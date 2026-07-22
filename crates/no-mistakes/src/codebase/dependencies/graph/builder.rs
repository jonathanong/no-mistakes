pub struct DepGraph {
    root: PathBuf,
    /// Base canonical graph built from source facts. Vitest setup edges stay
    /// compact until a graph traversal requests adjacency.
    edges: EdgeIndex<NodeId, EdgeKind>,
    vitest_setup_projects: Vec<VitestSetupProject>,
    effective_edges: OnceLock<EdgeIndex<NodeId, EdgeKind>>,
    parse_errors: HashMap<PathBuf, String>,
    resource_edge_details: ResourceEdgeDetails,
    resource_diagnostics: Vec<ResourceGraphDiagnostic>,
}

/// Compact Vitest project ownership retained until test-edge traversal.
pub(crate) struct VitestSetupProject {
    pub(crate) config: Option<String>,
    pub(crate) scope: Option<String>,
    pub(crate) filter: crate::codebase::test_discovery::ProjectTestFilter,
    pub(crate) setups: Vec<(PathBuf, VitestSetupField)>,
}

#[derive(Clone, Copy)]
enum SuppliedFactPolicy {
    FillSparse,
    RequireComplete,
}

pub(crate) struct PreparedGraphBuild<'a> {
    pub(crate) root: &'a Path,
    pub(crate) tsconfig: &'a TsConfig,
    pub(crate) tsconfig_catalog: Option<&'a crate::codebase::ts_resolver::TsConfigCatalog>,
    pub(crate) plan: GraphBuildPlan,
    pub(crate) graph_files: &'a GraphFiles,
    pub(crate) config_path: Option<&'a Path>,
    pub(crate) prepared: &'a PreparedGraphConfig,
    pub(crate) facts: Option<&'a dyn TsFactLookup>,
    pub(crate) import_resolution_cache:
        Option<&'a crate::codebase::ts_resolver::ImportResolutionCache>,
    pub(crate) dotnet_facts: Option<&'a crate::codebase::dotnet::DotnetFactMap>,
    pub(crate) swift_facts: Option<&'a crate::codebase::swift::SwiftFactMap>,
    pub(crate) visible_paths: Option<&'a crate::codebase::ts_source::VisiblePathSnapshot>,
}

impl DepGraph {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn build_with_plan_files_prepared_config_and_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
        config_path: Option<&Path>,
        prepared: &PreparedGraphConfig,
        facts: Option<&dyn TsFactLookup>,
    ) -> Result<Self> {
        Self::build_with_plan_files_prepared_config_facts_and_session(
            PreparedGraphBuildRequest {
                root,
                tsconfig,
                plan,
                graph_files,
                config_path,
                prepared,
                facts,
            },
            crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current()),
        )
    }

    pub(crate) fn build_with_plan_files_prepared_config_facts_and_resolution_cache(
        input: PreparedGraphBuild<'_>,
    ) -> Result<Self> {
        Self::build_with_plan_files_prepared_config_facts_resolution_cache_and_session(
            input,
            crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current()),
        )
    }

    pub(crate) fn build_with_plan_files_prepared_config_facts_resolution_cache_and_session(
        input: PreparedGraphBuild<'_>,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    ) -> Result<Self> {
        let PreparedGraphBuild {
            root,
            tsconfig,
            tsconfig_catalog,
            plan,
            graph_files,
            config_path,
            prepared,
            facts,
            import_resolution_cache,
            dotnet_facts,
            swift_facts,
            visible_paths,
        } = input;
        Self::build_with_plan_files_options_and_facts(
            GraphEdgeBuildInputs {
                root,
                tsconfig,
                tsconfig_catalog,
                plan,
                graph_files,
                workspace: Some(prepared.workspace()),
                config_options: prepared.options.as_ref(),
                playwright_settings: prepared.playwright_settings.as_ref(),
                config_path,
                dotnet_facts,
                swift_facts,
                import_resolution_cache,
                visible_paths,
            },
            facts,
            SuppliedFactPolicy::RequireComplete,
            session,
        )
    }

    pub(crate) fn build_with_plan_files_prepared_config_and_swift_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
        config_path: Option<&Path>,
        prepared: &PreparedGraphConfig,
        swift_facts: &crate::codebase::swift::SwiftFactMap,
    ) -> Result<Self> {
        Self::build_with_plan_files_options_and_facts(
            GraphEdgeBuildInputs {
                root,
                tsconfig,
                tsconfig_catalog: None,
                plan,
                graph_files,
                workspace: Some(prepared.workspace()),
                config_options: prepared.options.as_ref(),
                playwright_settings: prepared.playwright_settings.as_ref(),
                config_path,
                dotnet_facts: None,
                swift_facts: Some(swift_facts),
                import_resolution_cache: None,
                visible_paths: None,
            },
            None,
            SuppliedFactPolicy::RequireComplete,
            crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current()),
        )
    }

    pub(crate) fn build_with_plan_files_config_and_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
        config_path: Option<&Path>,
        facts: Option<&dyn TsFactLookup>,
    ) -> Result<Self> {
        let config_options = graph_config_options_for_plan_with_config(root, plan, config_path);
        Self::build_with_plan_files_options_and_facts(
            GraphEdgeBuildInputs {
                root,
                tsconfig,
                tsconfig_catalog: None,
                plan,
                graph_files,
                workspace: None,
                config_options: config_options.as_ref(),
                playwright_settings: None,
                config_path,
                dotnet_facts: None,
                swift_facts: None,
                import_resolution_cache: None,
                visible_paths: None,
            },
            facts,
            SuppliedFactPolicy::FillSparse,
            crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current()),
        )
    }
}
