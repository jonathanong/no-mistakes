impl DepGraph {
    pub(crate) fn build_with_plan_files_config_facts_and_session(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
        config_path: Option<&Path>,
        facts: Option<&dyn TsFactLookup>,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    ) -> Result<Self> {
        let config_options = graph_config_options_for_plan_with_config_and_session(
            root,
            plan,
            config_path,
            Some(&session),
            Some(graph_files.all()),
        );
        Self::build_with_plan_files_options_and_facts(
            GraphEdgeBuildInputs {
                root,
                tsconfig,
                tsconfig_catalog: None,
                plan,
                graph_files,
                workspace: None,
                config_options: config_options.as_ref(),
                playwright_settings: &[],
                config_path,
                dotnet_facts: None,
                swift_facts: None,
                import_resolution_cache: None,
                visible_paths: None,
                workflow_documents: None,
                interner: session.interner_arc(),
            },
            facts,
            SuppliedFactPolicy::FillSparse,
            session,
        )
    }
}
