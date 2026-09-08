use anyhow::{Context, Result};
use no_mistakes::codebase::check_facts::PlaywrightFactPlan;
use no_mistakes::codebase::dependencies::graph::GraphBuildPlan;
use no_mistakes::config::v2::NoMistakesConfig;
use no_mistakes::playwright::rules::{PlaywrightFactConsumers, PreparedPlaywrightRules};
use std::path::Path;

pub(super) fn fact_plan(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    canonical_graph_plan: Option<GraphBuildPlan>,
    prepared: Option<&PreparedPlaywrightRules>,
) -> Result<Option<PlaywrightFactPlan>> {
    let consumers = canonical_graph_plan
        .map(|plan| PlaywrightFactConsumers {
            graph_selectors: plan.playwright_selectors,
            graph_routes: plan.playwright_routes,
        })
        .unwrap_or_default();
    match prepared {
        Some(prepared) => Ok(Some(prepared.fact_plan())),
        None => no_mistakes::playwright::rules::fact_plan_for_consumers(
            root,
            config_path,
            config,
            consumers,
        )
        .context("failed to prepare Playwright shared facts"),
    }
}
