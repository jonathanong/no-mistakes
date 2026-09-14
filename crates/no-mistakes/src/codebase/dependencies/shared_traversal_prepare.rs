pub(crate) struct SnapshotTraversalPreparation {
    pub(crate) visible_paths: std::sync::Arc<crate::codebase::ts_source::VisiblePathSnapshot>,
    pub(crate) session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    pub(crate) include_check_plan: bool,
    pub(crate) needs_reverse_graph: bool,
    pub(crate) framework_plan: crate::codebase::test_discovery::FrameworkPreparationPlan,
}

struct TraversalPreparationContext {
    dataset: std::sync::Arc<crate::codebase::analysis_dataset::AnalysisDataset>,
    session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    include_check_plan: bool,
    needs_reverse_graph: bool,
    framework_plan: crate::codebase::test_discovery::FrameworkPreparationPlan,
}

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
            false,
        )
    }

    pub(crate) fn prepare_with_framework_plan_for_direction(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
        framework_plan: crate::codebase::test_discovery::FrameworkPreparationPlan,
        direction: Direction,
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
            matches!(direction, Direction::Dependents),
        )
    }

    fn prepare_with_session_and_framework_plan(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
        framework_plan: crate::codebase::test_discovery::FrameworkPreparationPlan,
        needs_reverse_graph: bool,
    ) -> Result<Self> {
        let dataset = session.dataset(&root);
        Self::prepare_with_dataset_session_and_framework_plan(
            root,
            tsconfig_path,
            config_path,
            build_plan,
            TraversalPreparationContext {
                dataset,
                session,
                include_check_plan: false,
                needs_reverse_graph,
                framework_plan,
            },
        )
    }

    pub(crate) fn prepare_with_snapshot_session_check_and_framework_plan(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
        preparation: SnapshotTraversalPreparation,
    ) -> Result<Self> {
        let SnapshotTraversalPreparation {
            visible_paths,
            session,
            include_check_plan,
            needs_reverse_graph,
            framework_plan,
        } = preparation;
        session.insert_visible_paths(&root, visible_paths);
        let dataset = session.dataset(&root);
        Self::prepare_with_dataset_session_and_framework_plan(
            root,
            tsconfig_path,
            config_path,
            build_plan,
            TraversalPreparationContext {
                dataset,
                session,
                include_check_plan,
                needs_reverse_graph,
                framework_plan,
            },
        )
    }
}

include!("shared_traversal_prepare_core.rs");
