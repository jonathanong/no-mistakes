#[derive(Default)]
struct PreparedGraphOptions {
    test_filter: Option<crate::codebase::test_filter::TestFileFilter>,
    workspace: Option<std::sync::Arc<crate::codebase::workspaces::IndexedWorkspaceMap>>,
    partial: bool,
}

#[doc(hidden)]
pub fn prepare_graph_config(
    root: &Path,
    plan: GraphBuildPlan,
    codebase_config: &crate::codebase::config::Config,
    config: &crate::config::v2::NoMistakesConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
) -> anyhow::Result<PreparedGraphConfig> {
    prepare_graph_config_inner(
        root,
        plan,
        codebase_config,
        config,
        visible_paths,
        PreparedGraphOptions::default(),
    )
}

#[doc(hidden)]
pub fn prepare_graph_config_with_test_filter(
    root: &Path,
    plan: GraphBuildPlan,
    codebase_config: &crate::codebase::config::Config,
    config: &crate::config::v2::NoMistakesConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    test_filter: crate::codebase::test_filter::TestFileFilter,
) -> anyhow::Result<PreparedGraphConfig> {
    prepare_graph_config_inner(
        root,
        plan,
        codebase_config,
        config,
        visible_paths,
        PreparedGraphOptions {
            test_filter: Some(test_filter),
            ..Default::default()
        },
    )
}
pub(crate) fn prepare_graph_config_with_test_filter_and_workspace(
    root: &Path,
    plan: GraphBuildPlan,
    codebase_config: &crate::codebase::config::Config,
    config: &crate::config::v2::NoMistakesConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    test_filter: crate::codebase::test_filter::TestFileFilter,
    workspace: std::sync::Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
) -> anyhow::Result<PreparedGraphConfig> {
    prepare_graph_config_inner(
        root,
        plan,
        codebase_config,
        config,
        visible_paths,
        PreparedGraphOptions {
            test_filter: Some(test_filter),
            workspace: Some(workspace),
            partial: false,
        },
    )
}

#[doc(hidden)]
pub fn prepare_graph_config_with_test_filter_partial(
    root: &Path,
    plan: GraphBuildPlan,
    codebase_config: &crate::codebase::config::Config,
    config: &crate::config::v2::NoMistakesConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    test_filter: crate::codebase::test_filter::TestFileFilter,
) -> anyhow::Result<PreparedGraphConfig> {
    prepare_graph_config_inner(
        root,
        plan,
        codebase_config,
        config,
        visible_paths,
        PreparedGraphOptions {
            test_filter: Some(test_filter),
            partial: true,
            ..Default::default()
        },
    )
}
