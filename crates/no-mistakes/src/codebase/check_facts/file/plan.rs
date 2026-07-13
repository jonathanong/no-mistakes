use super::{CheckFactPlan, PlaywrightFactPlan, TsFileFacts};
use std::path::Path;

pub(super) fn should_store_source(plan: &CheckFactPlan) -> bool {
    plan.source || plan.raw_source
}

pub(super) fn ts_source(source: Option<String>) -> TsFileFacts {
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
        || !plan.graph.is_empty()
        || match playwright {
            Some(plan) => plan.file(path).is_some() || plan.contains_source(path),
            None => false,
        }
        || plan.source
        || (!plan.raw_source && playwright.is_none())
}
