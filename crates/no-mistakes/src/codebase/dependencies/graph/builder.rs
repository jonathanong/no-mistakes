pub struct DepGraph {
    root: PathBuf,
    edges: EdgeIndex<NodeId, EdgeKind>,
    parse_errors: HashMap<PathBuf, String>,
}

#[derive(Clone, Copy)]
enum SuppliedFactPolicy {
    FillSparse,
    RequireComplete,
}

pub(crate) struct PreparedGraphBuild<'a> {
    pub(crate) root: &'a Path,
    pub(crate) tsconfig: &'a TsConfig,
    pub(crate) plan: GraphBuildPlan,
    pub(crate) graph_files: &'a GraphFiles,
    pub(crate) config_path: Option<&'a Path>,
    pub(crate) prepared: &'a PreparedGraphConfig,
    pub(crate) facts: Option<&'a dyn TsFactLookup>,
    pub(crate) import_resolution_cache:
        Option<&'a crate::codebase::ts_resolver::ImportResolutionCache>,
}

impl DepGraph {
    /// Build from request-scoped files and config that the caller prepared
    /// before entering a multi-consumer analysis fanout.
    pub(crate) fn build_with_plan_files_prepared_config(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
        config_path: Option<&Path>,
        prepared: &PreparedGraphConfig,
    ) -> Result<Self> {
        Self::build_with_plan_files_prepared_config_and_facts(
            root,
            tsconfig,
            plan,
            graph_files,
            config_path,
            prepared,
            None,
        )
    }

    pub(crate) fn build_with_plan_files_prepared_config_and_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
        config_path: Option<&Path>,
        prepared: &PreparedGraphConfig,
        facts: Option<&dyn TsFactLookup>,
    ) -> Result<Self> {
        Self::build_with_plan_files_prepared_config_facts_and_resolution_cache(PreparedGraphBuild {
            root,
            tsconfig,
            plan,
            graph_files,
            config_path,
            prepared,
            facts,
            import_resolution_cache: None,
        })
    }

    pub(crate) fn build_with_plan_files_prepared_config_facts_and_resolution_cache(
        input: PreparedGraphBuild<'_>,
    ) -> Result<Self> {
        let PreparedGraphBuild {
            root,
            tsconfig,
            plan,
            graph_files,
            config_path,
            prepared,
            facts,
            import_resolution_cache,
        } = input;
        Self::build_with_plan_files_options_and_facts(
            GraphEdgeBuildInputs {
                root,
                tsconfig,
                plan,
                graph_files,
                workspace: Some(prepared.workspace()),
                config_options: prepared.options.as_ref(),
                playwright_settings: prepared.playwright_settings.as_ref(),
                config_path,
                swift_facts: None,
                import_resolution_cache,
            },
            facts,
            SuppliedFactPolicy::RequireComplete,
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
                plan,
                graph_files,
                workspace: Some(prepared.workspace()),
                config_options: prepared.options.as_ref(),
                playwright_settings: prepared.playwright_settings.as_ref(),
                config_path,
                swift_facts: Some(swift_facts),
                import_resolution_cache: None,
            },
            None,
            SuppliedFactPolicy::RequireComplete,
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
                plan,
                graph_files,
                workspace: None,
                config_options: config_options.as_ref(),
                playwright_settings: None,
                config_path,
                swift_facts: None,
                import_resolution_cache: None,
            },
            facts,
            SuppliedFactPolicy::FillSparse,
        )
    }

    pub(crate) fn build_with_plan_file_list_prepared_config_and_check_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        files: Vec<PathBuf>,
        config_path: Option<&Path>,
        facts: &crate::codebase::check_facts::CheckFactMap,
        prepared: &PreparedGraphConfig,
    ) -> Result<Self> {
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
                swift_facts: None,
                import_resolution_cache: None,
            },
            Some(facts as &dyn TsFactLookup),
            SuppliedFactPolicy::RequireComplete,
        )
    }

    pub(crate) fn build_with_plan_file_list_config_and_complete_check_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        files: Vec<PathBuf>,
        config_path: Option<&Path>,
        facts: &crate::codebase::check_facts::CheckFactMap,
    ) -> Result<Self> {
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
                swift_facts: None,
                import_resolution_cache: None,
            },
            Some(facts as &dyn TsFactLookup),
            SuppliedFactPolicy::RequireComplete,
        )
    }
}
