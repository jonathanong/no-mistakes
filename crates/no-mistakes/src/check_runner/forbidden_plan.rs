use anyhow::Result;
use no_mistakes::codebase::check_facts::{CheckFactPlan, PlaywrightFactPlan};
use no_mistakes::codebase::dependencies::graph::{GraphBuildPlan, PreparedGraphConfig};
use no_mistakes::codebase::ts_resolver::TsConfig;
use no_mistakes::codebase::ts_source::VisiblePathSnapshot;
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::Path;

pub(super) struct PreparedInputs<'a> {
    pub(super) codebase_config: &'a no_mistakes::codebase::config::Config,
    pub(super) tsconfig: &'a TsConfig,
    pub(super) visible_paths: &'a VisiblePathSnapshot,
}

pub(super) fn prepare(
    root: &Path,
    config: &NoMistakesConfig,
    inputs: PreparedInputs<'_>,
    graph_plan: Option<GraphBuildPlan>,
    playwright_fact_plan: &mut Option<PlaywrightFactPlan>,
    plan: &mut CheckFactPlan,
) -> Result<Option<PreparedGraphConfig>> {
    let prepared_graph = graph_plan
        .map(|graph_plan| {
            no_mistakes::codebase::dependencies::graph::prepare_graph_config(
                root,
                graph_plan,
                inputs.codebase_config,
                config,
                inputs.visible_paths,
            )
        })
        .transpose()?;
    if let Some(graph_playwright) = prepared_graph
        .as_ref()
        .map(|graph| graph.playwright_fact_plan(root, inputs.tsconfig, inputs.visible_paths))
        .transpose()?
        .flatten()
    {
        match playwright_fact_plan.as_mut() {
            Some(plan) => plan.include(graph_playwright),
            None => *playwright_fact_plan = Some(graph_playwright),
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
    Ok(prepared_graph)
}
