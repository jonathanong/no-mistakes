pub(crate) struct PreparedGraphFacts<'a> {
    pub(crate) ts: Option<&'a dyn TsFactLookup>,
    pub(crate) dotnet: Option<&'a crate::codebase::dotnet::DotnetFactMap>,
    pub(crate) swift: Option<&'a crate::codebase::swift::SwiftFactMap>,
}

pub(crate) struct PreparedGraphBuildRequest<'a> {
    pub(crate) root: &'a Path,
    pub(crate) tsconfig: &'a TsConfig,
    pub(crate) plan: GraphBuildPlan,
    pub(crate) graph_files: &'a GraphFiles,
    pub(crate) config_path: Option<&'a Path>,
    pub(crate) prepared: &'a PreparedGraphConfig,
    pub(crate) facts: Option<&'a dyn TsFactLookup>,
}

pub(crate) struct PreparedCheckFactGraphBuildRequest<'a> {
    pub(crate) root: &'a Path,
    pub(crate) tsconfig: &'a TsConfig,
    pub(crate) plan: GraphBuildPlan,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) config_path: Option<&'a Path>,
    pub(crate) facts: &'a crate::codebase::check_facts::CheckFactMap,
    pub(crate) prepared: &'a PreparedGraphConfig,
}

pub(crate) struct CompleteCheckFactGraphBuildRequest<'a> {
    pub(crate) root: &'a Path,
    pub(crate) tsconfig: &'a TsConfig,
    pub(crate) plan: GraphBuildPlan,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) config_path: Option<&'a Path>,
    pub(crate) facts: &'a crate::codebase::check_facts::CheckFactMap,
}

impl DepGraph {
    pub(crate) fn build_with_plan_files_prepared_config_facts_and_session(
        request: PreparedGraphBuildRequest<'_>,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    ) -> Result<Self> {
        let PreparedGraphBuildRequest {
            root,
            tsconfig,
            plan,
            graph_files,
            config_path,
            prepared,
            facts,
        } = request;
        Self::build_with_plan_files_prepared_config_facts_resolution_cache_and_session(
            PreparedGraphBuild {
                root,
                tsconfig,
                plan,
                graph_files,
                config_path,
                prepared,
                facts,
                import_resolution_cache: None,
                dotnet_facts: None,
                swift_facts: None,
                visible_paths: None,
            },
            session,
        )
    }

    pub(crate) fn build_with_plan_files_prepared_config_and_all_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
        config_path: Option<&Path>,
        prepared: &PreparedGraphConfig,
        facts: PreparedGraphFacts<'_>,
    ) -> Result<Self> {
        Self::build_with_plan_files_prepared_config_facts_and_resolution_cache(PreparedGraphBuild {
            root,
            tsconfig,
            plan,
            graph_files,
            config_path,
            prepared,
            facts: facts.ts,
            import_resolution_cache: None,
            dotnet_facts: facts.dotnet,
            swift_facts: facts.swift,
            visible_paths: None,
        })
    }

    pub(crate) fn build_with_prepared_check_facts_and_session(
        request: PreparedCheckFactGraphBuildRequest<'_>,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    ) -> Result<Self> {
        let PreparedCheckFactGraphBuildRequest {
            root,
            tsconfig,
            plan,
            files,
            config_path,
            facts,
            prepared,
        } = request;
        let graph_files = GraphFiles::from_files(files);
        Self::build_with_plan_files_options_and_facts(
            GraphEdgeBuildInputs {
                root,
                tsconfig,
                plan,
                graph_files: &graph_files,
                workspace: Some(prepared.workspace()),
                config_options: prepared.options.as_ref(),
                playwright_settings: prepared.playwright_settings.as_ref(),
                config_path,
                dotnet_facts: None,
                swift_facts: None,
                import_resolution_cache: None,
                visible_paths: None,
            },
            Some(facts as &dyn TsFactLookup),
            SuppliedFactPolicy::RequireComplete,
            session,
        )
    }

    pub(crate) fn build_with_complete_check_facts_and_session(
        request: CompleteCheckFactGraphBuildRequest<'_>,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    ) -> Result<Self> {
        let CompleteCheckFactGraphBuildRequest {
            root,
            tsconfig,
            plan,
            files,
            config_path,
            facts,
        } = request;
        let graph_files = GraphFiles::from_files(files);
        let config_options = graph_config_options_for_plan_with_config(root, plan, config_path);
        Self::build_with_plan_files_options_and_facts(
            GraphEdgeBuildInputs {
                root,
                tsconfig,
                plan,
                graph_files: &graph_files,
                workspace: None,
                config_options: config_options.as_ref(),
                playwright_settings: None,
                config_path,
                dotnet_facts: None,
                swift_facts: None,
                import_resolution_cache: None,
                visible_paths: None,
            },
            Some(facts as &dyn TsFactLookup),
            SuppliedFactPolicy::RequireComplete,
            session,
        )
    }
}
