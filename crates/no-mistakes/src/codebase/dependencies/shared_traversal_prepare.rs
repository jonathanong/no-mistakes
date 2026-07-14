impl SharedTraversalContext {
    pub(crate) fn prepare_with_framework_plan(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
        framework_plan: crate::codebase::test_discovery::FrameworkPreparationPlan,
    ) -> Result<Self> {
        let session =
            crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
        Self::prepare_with_session_and_framework_plan(
            root,
            tsconfig_path,
            config_path,
            build_plan,
            session,
            framework_plan,
        )
    }

    pub(crate) fn prepare_with_session(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    ) -> Result<Self> {
        let framework_plan =
            crate::codebase::test_discovery::FrameworkPreparationPlan::for_graph(build_plan);
        Self::prepare_with_session_and_framework_plan(
            root,
            tsconfig_path,
            config_path,
            build_plan,
            session,
            framework_plan,
        )
    }

    fn prepare_with_session_and_framework_plan(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
        framework_plan: crate::codebase::test_discovery::FrameworkPreparationPlan,
    ) -> Result<Self> {
        let dataset = session.dataset(&root);
        Self::prepare_with_dataset_session_and_framework_plan(
            root,
            tsconfig_path,
            config_path,
            build_plan,
            dataset,
            session,
            false,
            framework_plan,
        )
    }

    pub(crate) fn prepare_with_snapshot_session_check_and_framework_plan(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
        visible_paths: std::sync::Arc<crate::codebase::ts_source::VisiblePathSnapshot>,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
        include_check_plan: bool,
        framework_plan: crate::codebase::test_discovery::FrameworkPreparationPlan,
    ) -> Result<Self> {
        session.insert_visible_paths(&root, visible_paths);
        let dataset = session.dataset(&root);
        Self::prepare_with_dataset_session_and_framework_plan(
            root,
            tsconfig_path,
            config_path,
            build_plan,
            dataset,
            session,
            include_check_plan,
            framework_plan,
        )
    }
}

include!("shared_traversal_prepare_core.rs");
