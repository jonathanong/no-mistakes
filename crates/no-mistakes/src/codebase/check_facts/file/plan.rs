use super::{CheckFactPlan, PlaywrightFactPlan, TsFileFacts};
use crate::codebase::ts_source::facts::{TsFactContext, TsFactPlan};
use std::path::Path;

#[cfg(test)]
mod tests;

pub(super) fn should_store_source(plan: &CheckFactPlan) -> bool {
    plan.source || plan.raw_source
}

/// Graph/check TS extract demand for one file. Playwright test files also need
/// import facts even when the shared plan did not request them.
pub(super) fn ts_extract_plan(
    plan: &CheckFactPlan,
    path: &Path,
    playwright: Option<&PlaywrightFactPlan>,
) -> TsFactPlan {
    let mut ts_plan = plan.collected_ts_plan();
    if playwright.is_some_and(|plan| plan.file(path).is_some()) {
        ts_plan.include(TsFactPlan::imports());
    }
    ts_plan
}

/// Context for the shared TS extract. Check-only queue factory names override
/// the graph context so `ts.queue_project` stays aligned with the check plan.
pub(super) fn ts_extract_context(root: &Path, plan: &CheckFactPlan) -> TsFactContext {
    let mut context = plan.graph_context.clone();
    context.root = root.to_path_buf();
    if plan.queue || plan.graph.queue_project {
        context.queue_project_factory_names = plan.queue_factory_names.clone();
    }
    context
}

pub(super) fn ts_source(source: Option<std::sync::Arc<str>>) -> TsFileFacts {
    TsFileFacts {
        source,
        route_helpers: Vec::new(),
        route_helper_imports: Vec::new(),
        route_helper_refs: Vec::new(),
        ..Default::default()
    }
}

pub(super) fn requires_parse(
    plan: &CheckFactPlan,
    path: &Path,
    playwright: Option<&PlaywrightFactPlan>,
) -> bool {
    plan.imports
        || plan.symbols
        || plan.react
        || plan.react_usages
        || plan.queue
        || plan.integration
        || plan
            .integration_runner_configs
            .as_ref()
            .is_some_and(|runner| runner.contains(path))
        || plan.dynamic_imports
        || plan.nextjs_caching
        || plan.storybook
        || plan.server_route_client_boundary
        || !plan.graph.is_empty()
        || match playwright {
            Some(plan) => plan.file(path).is_some() || plan.contains_source(path),
            None => false,
        }
        || plan.source
        || (!plan.raw_source && playwright.is_none())
}
