/// Config-derived graph settings prepared once for an aggregate check request.
#[doc(hidden)]
pub struct PreparedGraphConfig {
    options: Option<GraphConfigOptions>,
    playwright_settings: Vec<crate::playwright::config::Settings>,
    workspace: std::sync::Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
    workflow_documents:
        Option<std::sync::Arc<crate::codebase::ci_workflows::ParsedWorkflowSet>>,
}

impl PreparedGraphConfig {
    /// Build the Playwright fact scope required by prepared graph edge
    /// producers, merging one fact plan per resolved frontend app so a
    /// multi-app repository's route/selector facts all get collected
    /// instead of only the arbitrarily-first app's.
    #[doc(hidden)]
    pub fn playwright_fact_plan(
        &self,
        root: &Path,
        tsconfig: &crate::codebase::ts_resolver::TsConfig,
        visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    ) -> anyhow::Result<Option<crate::codebase::check_facts::PlaywrightFactPlan>> {
        let mut merged: Option<crate::codebase::check_facts::PlaywrightFactPlan> = None;
        for settings in &self.playwright_settings {
            let plan = merged.get_or_insert_with(Default::default);
            crate::playwright::analysis::pipeline::extend_standalone_fact_plan(
                plan,
                root,
                settings,
                crate::playwright::analysis::types::UniqueSelectorPolicy::default(),
                visible_paths,
            )?;
        }
        if let Some(plan) = merged.as_mut() {
            plan.configure_module_resolution(
                std::sync::Arc::new(tsconfig.clone()),
                std::sync::Arc::clone(&self.workspace),
                visible_paths,
                root,
            );
        }
        Ok(merged)
    }
    pub(crate) fn workspace(&self) -> &crate::codebase::workspaces::IndexedWorkspaceMap {
        self.workspace.as_ref()
    }

    pub(crate) fn workflow_documents(
        &self,
    ) -> Option<&crate::codebase::ci_workflows::ParsedWorkflowSet> {
        self.workflow_documents.as_deref()
    }

    /// Supply request-prepared workflow documents for graph projections.
    #[doc(hidden)]
    pub fn set_workflow_documents(
        &mut self,
        documents: Option<std::sync::Arc<crate::codebase::ci_workflows::ParsedWorkflowSet>>,
    ) {
        self.workflow_documents = documents;
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
        graph_config_options_from_loaded_with_test_filter(
            root,
            codebase_config,
            config,
            &visible_paths.paths_for(root),
            test_filter,
        )
    });
    // One `Settings` per resolved frontend app, so a multi-`type: nextjs`-app
    // repository builds route/selector edges for every app instead of
    // erroring (no `cli_project` exists at this scope to bind an ambiguous
    // app against) or silently using only whichever app sorted first (#624).
    // `frontend_apps_or_default` never returns empty, so the pre-#624/#625
    // zero-signal fallback (`default_frontend_root`) still applies via the
    // single synthetic entry it produces.
    //
    // Resolving apps is skipped entirely when `has_v2_playwright_settings` is
    // false: every per-app `settings_from_loaded_v2` call would collapse to
    // the same app-agnostic `settings_from_defaults` fallback regardless of
    // which app (if any) is named, so resolving the app set first would only
    // discard the result unused — wasted work for every non-Playwright repo
    // that still configures `type: nextjs` projects for other reasons.
    let playwright_settings = if !(plan.playwright_routes || plan.playwright_selectors) {
        Vec::new()
    } else if crate::playwright::config::has_v2_playwright_settings(config) {
        let apps = crate::config::v2::frontend_apps_or_default(
            root,
            config,
            &visible_paths.paths_for(root),
        )?;
        apps.iter()
            .map(|app| {
                crate::playwright::config::settings_from_loaded_v2(
                    root,
                    config,
                    &[],
                    None,
                    app.project.clone(),
                    visible_paths,
                )
            })
            .collect::<anyhow::Result<Vec<_>>>()?
    } else {
        vec![crate::playwright::config::settings_from_loaded_v2(
            root,
            config,
            &[],
            None,
            None,
            visible_paths,
        )?]
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
        workflow_documents: None,
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
