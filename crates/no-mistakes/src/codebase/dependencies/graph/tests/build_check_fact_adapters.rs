
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
        Self::build_with_prepared_check_facts_and_session(
            PreparedCheckFactGraphBuildRequest {
                root,
                tsconfig,
                plan,
                files,
                config_path,
                facts,
                prepared,
            },
            crate::codebase::analysis_session::AnalysisSession::new(
                crate::diagnostics::current(),
            ),
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
        Self::build_with_complete_check_facts_and_session(
            CompleteCheckFactGraphBuildRequest {
                root,
                tsconfig,
                plan,
                files,
                config_path,
                facts,
            },
            crate::codebase::analysis_session::AnalysisSession::new(
                crate::diagnostics::current(),
            ),
        )
    }

}
