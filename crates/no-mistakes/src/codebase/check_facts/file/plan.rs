use super::{CheckFactPlan, PlaywrightFactPlan, TsFileFacts};
use crate::codebase::ts_source::facts::{TsFactContext, TsFactPlan};
use std::path::Path;

#[cfg(test)]
mod tests;

pub(super) fn should_store_source(plan: &CheckFactPlan, path: &Path) -> bool {
    // React checks parse-error suppression before filtering target files.
    plan.source
        || (plan.raw_source
            && (plan.react
                || plan.graph.source
                || crate::codebase::check_facts::family_scope::applies(
                    &plan.raw_source_paths,
                    path,
                )))
}

/// Graph/check TS extract demand for one file. Playwright test files also need
/// import facts even when the shared plan did not request them.
pub(super) fn ts_extract_plan(
    root: &Path,
    plan: &CheckFactPlan,
    path: &Path,
    playwright: Option<&PlaywrightFactPlan>,
) -> TsFactPlan {
    let mut ts_plan = plan.collected_ts_plan();
    ts_plan.source = plan.graph.source || should_store_source(plan, path);
    ts_plan.call_sites = plan.requires_call_sites(path);
    if !ts_plan.call_arguments {
        ts_plan.call_arguments = plan
            .forbidden_call_arguments
            .as_ref()
            .is_some_and(|scope| scope.applies(path));
    }
    ts_plan.module_bindings |= super::sql_identifier::selected(root, path, plan);
    ts_plan.imports |= plan.production_import_paths.contains(path);
    if playwright.is_some_and(|plan| plan.file(path).is_some()) {
        ts_plan.include(TsFactPlan::imports());
    }
    if !plan.graph.react
        && plan.react
        && !crate::codebase::check_facts::family_scope::applies(&plan.react_paths, path)
    {
        ts_plan.react = false;
    }
    if !plan.graph.queue_project
        && plan.queue
        && !crate::codebase::check_facts::family_scope::applies(&plan.queue_paths, path)
    {
        ts_plan.queue_project = false;
    }
    if !plan.graph.symbols
        && plan.symbols
        && !crate::codebase::check_facts::family_scope::applies(&plan.symbols_paths, path)
    {
        ts_plan.symbols = false;
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
        context.queue_factory_roles = plan.queue_factory_roles.clone();
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
    root: &Path,
    plan: &CheckFactPlan,
    path: &Path,
    playwright: Option<&PlaywrightFactPlan>,
) -> bool {
    plan.requires_call_sites(path)
        || plan.imports
        || plan.production_import_paths.contains(path)
        || plan.query_annotation
        || plan.query_annotation_heads
        || super::sql_identifier::selected(root, path, plan)
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
        || plan.nextjs_caching_low
        || plan.storybook
        || plan.server_route_client_boundary
        || !plan.graph.is_empty()
        || plan
            .forbidden_call_arguments
            .as_ref()
            .is_some_and(|scope| scope.applies(path))
        || match playwright {
            Some(plan) => plan.file(path).is_some() || plan.contains_source(path),
            None => false,
        }
        || plan.source
        || (!plan.raw_source && playwright.is_none())
}
