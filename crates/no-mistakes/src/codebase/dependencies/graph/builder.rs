pub struct DepGraph {
    root: PathBuf,
    /// forward: node → nodes it imports/references (with edge kinds)
    forward: EdgeMap,
    /// reverse: node → nodes that import/reference it (with edge kinds)
    reverse: EdgeMap,
    parse_errors: HashMap<PathBuf, String>,
}

#[derive(Clone, Copy)]
enum SuppliedFactPolicy {
    FillSparse,
    RequireComplete,
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
        Self::build_with_plan_files_options_and_facts(
            GraphEdgeBuildInputs {
                root,
                tsconfig,
                plan,
                graph_files,
                config_options: prepared.options.as_ref(),
                playwright_settings: prepared.playwright_settings.as_ref(),
                config_path,
                swift_facts: None,
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
                config_options: prepared.options.as_ref(),
                playwright_settings: prepared.playwright_settings.as_ref(),
                config_path,
                swift_facts: Some(swift_facts),
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
                config_options: config_options.as_ref(),
                playwright_settings: None,
                config_path,
                swift_facts: None,
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
                config_options: prepared.options.as_ref(),
                playwright_settings: prepared.playwright_settings.as_ref(),
                config_path,
                swift_facts: None,
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
                config_options: config_options.as_ref(),
                playwright_settings: None,
                config_path,
                swift_facts: None,
            },
            Some(facts as &dyn TsFactLookup),
            SuppliedFactPolicy::RequireComplete,
        )
    }
}
