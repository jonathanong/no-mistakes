use anyhow::Result;
use no_mistakes::codebase::check_facts::{CheckFactPlan, PlaywrightFactPlan};
use no_mistakes::codebase::dependencies::graph::{GraphBuildPlan, PreparedGraphConfig};
use no_mistakes::codebase::ts_resolver::TsConfig;
use no_mistakes::codebase::ts_source::VisiblePathSnapshot;
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::Path;

pub(crate) struct PreparedInputs<'a> {
    pub(crate) session: &'a no_mistakes::codebase::analysis_session::AnalysisSession,
    pub(crate) codebase_config: &'a no_mistakes::codebase::config::Config,
    pub(crate) tsconfig: &'a TsConfig,
    pub(crate) visible_paths: &'a VisiblePathSnapshot,
    pub(crate) workflow_documents: Option<
        &'a std::sync::Arc<no_mistakes::codebase::ci_workflows::PreparedWorkflowTopologyInputs>,
    >,
}

pub(crate) fn prepare(
    root: &Path,
    config: &NoMistakesConfig,
    inputs: PreparedInputs<'_>,
    graph_plan: Option<GraphBuildPlan>,
    playwright_fact_plan: &mut Option<PlaywrightFactPlan>,
    plan: &mut CheckFactPlan,
) -> Result<Option<PreparedGraphConfig>> {
    failures::initialize(inputs.session, config);
    let mut prepared_graph = graph_plan
        .map(|graph_plan| {
            let test_filter = no_mistakes::codebase::test_filter::TestFileFilter::new(root, config);
            let prepare = if inputs.session.partial_checks_enabled() {
                no_mistakes::codebase::dependencies::graph::prepare_graph_config_with_test_filter_partial
            } else {
                no_mistakes::codebase::dependencies::graph::prepare_graph_config_with_test_filter
            };
            let result = prepare(
                root,
                graph_plan,
                inputs.codebase_config,
                config,
                inputs.visible_paths,
                test_filter,
            );
            match result {
                Ok(value) => Ok(value),
                Err(error) if graph_plan.playwright_routes || graph_plan.playwright_selectors => {
                    failures::record(inputs.session, config, &error)?;
                    let mut fallback = graph_plan;
                    fallback.playwright_routes = false;
                    fallback.playwright_selectors = false;
                    let test_filter = no_mistakes::codebase::test_filter::TestFileFilter::new(root, config);
                    let mut prepared = no_mistakes::codebase::dependencies::graph::prepare_graph_config_with_test_filter(
                        root, fallback, inputs.codebase_config, config, inputs.visible_paths, test_filter)?;
                    prepared.disable_playwright();
                    Ok(prepared)
                }
                Err(error) => Err(error),
            }
        })
        .transpose()?;
    if let Some(prepared) = prepared_graph.as_mut() {
        prepared.set_workflow_inputs(inputs.workflow_documents.cloned());
    }
    if let Some(graph) = prepared_graph.as_mut() {
        let graph_playwright = if inputs.session.partial_checks_enabled() {
            let (plan, errors) =
                graph.playwright_fact_plan_partial(root, inputs.tsconfig, inputs.visible_paths);
            for error in errors {
                failures::record_prepared(
                    root,
                    inputs.session,
                    config,
                    inputs.visible_paths.paths_for(root).as_ref(),
                    &error,
                )?;
            }
            plan
        } else {
            graph.playwright_fact_plan(root, inputs.tsconfig, inputs.visible_paths)?
        };
        if let Some(graph_playwright) = graph_playwright {
            match playwright_fact_plan.as_mut() {
                Some(plan) => plan.include(graph_playwright),
                None => *playwright_fact_plan = Some(graph_playwright),
            }
        }
    }
    if let (Some(graph_plan), Some(prepared)) = (graph_plan, prepared_graph.as_ref()) {
        let (fact_plan, fact_context) =
            no_mistakes::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
                root,
                graph_plan,
                prepared,
            );
        plan.graph.include(fact_plan);
        plan.graph_context = fact_context;
    }
    plan.graph.per_item_calls |=
        config.rule_configured(no_mistakes::codebase::rules::QUERY_REACHED_PER_ITEM);
    no_mistakes::codebase::rules::forbidden_calls::configure_argument_facts(root, config, plan);
    Ok(prepared_graph)
}

pub(crate) mod failures;
