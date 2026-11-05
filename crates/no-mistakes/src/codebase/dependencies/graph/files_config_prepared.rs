/// Config-derived graph settings prepared once for an aggregate check request.
#[doc(hidden)]
pub struct PreparedGraphConfig {
    options: Option<GraphConfigOptions>,
    playwright_failed: bool,
    playwright_preparation_failures: Vec<PlaywrightPreparationFailure>,
    playwright_settings: Vec<crate::playwright::config::Settings>,
    workspace: std::sync::Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
    workflow_documents: Option<std::sync::Arc<crate::codebase::ci_workflows::ParsedWorkflowSet>>,
    workflow_inputs:
        Option<std::sync::Arc<crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs>>,
}

impl PreparedGraphConfig {
    #[doc(hidden)]
    pub fn disable_playwright(&mut self) {
        self.playwright_failed = true;
    }

    pub(crate) fn effective_plan(&self, mut plan: GraphBuildPlan) -> GraphBuildPlan {
        if self.playwright_failed {
            plan.playwright_routes = false;
            plan.playwright_selectors = false;
        }
        plan
    }

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
    ) -> Option<crate::codebase::ci_workflows::PreparedWorkflowDocuments<'_>> {
        use crate::codebase::ci_workflows::PreparedWorkflowDocuments;
        self.workflow_inputs
            .as_deref()
            .map(PreparedWorkflowDocuments::Topology)
            .or_else(|| {
                self.workflow_documents
                    .as_deref()
                    .map(PreparedWorkflowDocuments::Documents)
            })
    }

    #[doc(hidden)]
    pub fn set_workflow_inputs(
        &mut self,
        inputs: Option<
            std::sync::Arc<crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs>,
        >,
    ) {
        self.workflow_documents = None;
        self.workflow_inputs = inputs;
    }

    /// Supply request-prepared workflow documents for graph projections.
    #[doc(hidden)]
    pub fn set_workflow_documents(
        &mut self,
        documents: Option<std::sync::Arc<crate::codebase::ci_workflows::ParsedWorkflowSet>>,
    ) {
        self.workflow_inputs = None;
        self.workflow_documents = documents;
    }
}

include!("files_config_prepared/constructors.rs");

fn prepare_graph_config_inner(
    root: &Path,
    plan: GraphBuildPlan,
    codebase_config: &crate::codebase::config::Config,
    config: &crate::config::v2::NoMistakesConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    preparation: PreparedGraphOptions,
) -> anyhow::Result<PreparedGraphConfig> {
    let PreparedGraphOptions {
        test_filter,
        workspace,
        partial,
    } = preparation;
    let options = graph_plan_needs_config(plan).then(|| {
        graph_config_options_from_loaded_with_test_filter(
            root,
            codebase_config,
            config,
            &visible_paths.paths_for(root),
            test_filter,
        )
    });
    let (playwright_settings, playwright_preparation_failures) =
        prepare_playwright_settings(root, plan, config, visible_paths, partial)?;
    Ok(PreparedGraphConfig {
        options,
        playwright_failed: playwright_settings.is_empty()
            && !playwright_preparation_failures.is_empty(),
        playwright_preparation_failures,
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
        workflow_inputs: None,
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

include!("files_config_prepared/partial.rs");

include!("files_config_prepared/failure.rs");

include!("files_config_prepared/settings.rs");
