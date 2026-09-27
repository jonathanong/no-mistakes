use crate::codebase::check_facts::{CheckFactPlan, PlaywrightFactPlan};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::schema::NoMistakesConfig;
use std::path::Path;
use std::sync::Arc;

/// Add runner demand before collection, borrowing the same resolver and sources.
pub(crate) fn runner_plan(
    root: &Path,
    plan: &mut CheckFactPlan,
    playwright: &PlaywrightFactPlan,
    sources: &Arc<SourceStore>,
) {
    let settings = playwright.integration_route_settings().collect::<Vec<_>>();
    if settings.is_empty() || plan.integration_runner_configs.is_some() {
        return;
    }
    let Some(resolution) = playwright.module_resolution() else {
        return;
    };
    let mut config = NoMistakesConfig::default();
    config.tests.vitest = settings[0].route_coverage_vitest.clone();
    config.tests.playwright.route_coverage_sources = settings
        .iter()
        .flat_map(|settings| settings.route_coverage_sources.iter().cloned())
        .collect();
    plan.integration_runner_configs = Some(Arc::new(
        crate::integration_tests::prepare_runner_configs_with_catalog(
            root,
            &config,
            &sources.inventory().paths(),
            resolution.catalog(root),
            Arc::clone(sources),
        ),
    ));
}
