/// Config-derived graph settings prepared once for an aggregate check request.
#[doc(hidden)]
pub struct PreparedGraphConfig {
    options: Option<GraphConfigOptions>,
    playwright_settings: Option<crate::playwright::config::Settings>,
    workspace: std::sync::Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
}

impl PreparedGraphConfig {
    /// Build the Playwright fact scope required by prepared graph edge producers, if any.
    #[doc(hidden)]
    pub fn playwright_fact_plan(
        &self,
        root: &Path,
        tsconfig: &crate::codebase::ts_resolver::TsConfig,
        visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    ) -> anyhow::Result<Option<crate::codebase::check_facts::PlaywrightFactPlan>> {
        self.playwright_settings
            .as_ref()
            .map(|settings| {
                let mut plan = crate::playwright::analysis::pipeline::standalone_fact_plan(
                    root,
                    settings,
                    crate::playwright::analysis::types::UniqueSelectorPolicy::default(),
                    visible_paths,
                )?;
                plan.configure_module_resolution(
                    std::sync::Arc::new(tsconfig.clone()),
                    std::sync::Arc::clone(&self.workspace),
                    visible_paths,
                    root,
                );
                Ok(plan)
            })
            .transpose()
    }
    pub(crate) fn workspace(&self) -> &crate::codebase::workspaces::IndexedWorkspaceMap {
        self.workspace.as_ref()
    }

}

#[doc(hidden)]
pub fn prepare_graph_config(
    root: &Path,
    plan: GraphBuildPlan,
    codebase_config: &crate::codebase::config::Config,
    config: &crate::config::v2::NoMistakesConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
) -> anyhow::Result<PreparedGraphConfig> {
    prepare_graph_config_inner(root, plan, codebase_config, config, visible_paths, None, None)
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
        Some(test_filter),
        None,
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
        Some(test_filter),
        Some(workspace),
    )
}


fn prepare_graph_config_inner(
    root: &Path,
    plan: GraphBuildPlan,
    codebase_config: &crate::codebase::config::Config,
    config: &crate::config::v2::NoMistakesConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    test_filter: Option<crate::codebase::test_filter::TestFileFilter>,
    workspace: Option<std::sync::Arc<crate::codebase::workspaces::IndexedWorkspaceMap>>,
) -> anyhow::Result<PreparedGraphConfig> {
    let options = graph_plan_needs_config(plan).then(|| {
        graph_config_options_from_loaded_with_test_filter(root, codebase_config, config, test_filter)
    });
    let playwright_settings = if plan.playwright_routes || plan.playwright_selectors {
        Some(crate::playwright::config::settings_from_loaded_v2(
            root,
            config,
            &[],
            None,
            visible_paths,
        )?)
    } else {
        None
    };
    Ok(PreparedGraphConfig {
        options,
        playwright_settings,
        workspace: workspace.unwrap_or_else(|| {
            std::sync::Arc::new(
                crate::codebase::workspaces::load_indexed_from_source_store(
                    root,
                    &visible_paths.source_store_for(root),
                )
                .unwrap_or_default(),
            )
        }),
    })
}

#[doc(hidden)]
pub fn ts_fact_plan_and_context_for_plan_with_prepared(
    root: &Path,
    plan: GraphBuildPlan,
    prepared: &PreparedGraphConfig,
) -> (TsFactPlan, TsFactContext) {
    (
        effective_ts_fact_plan(plan, prepared.options.as_ref()),
        ts_fact_context_from_options(root, plan, prepared.options.as_ref()),
    )
}
