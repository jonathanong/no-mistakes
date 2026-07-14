impl DepGraph {
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
                dotnet_facts: None,
                swift_facts: None,
                import_resolution_cache: None,
                visible_paths: None,
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
                dotnet_facts: None,
                swift_facts: None,
                import_resolution_cache: None,
                visible_paths: None,
            },
            Some(facts as &dyn TsFactLookup),
            SuppliedFactPolicy::RequireComplete,
        )
    }
}
