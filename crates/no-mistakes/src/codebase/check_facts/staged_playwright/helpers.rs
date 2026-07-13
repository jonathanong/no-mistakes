use super::super::{collect_fact_map, CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn collect_test_partition(
    root: &Path,
    files: &[PathBuf],
    plan: CheckFactPlan,
    playwright: &PlaywrightFactPlan,
    facts: &mut HashMap<PathBuf, CheckFileFacts>,
) {
    let files = files
        .iter()
        .filter(|path| !facts.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    facts.extend(collect_fact_map(root, &files, &plan, Some(playwright)));
}

pub(super) fn with_imports(mut plan: CheckFactPlan) -> CheckFactPlan {
    plan.graph.imports = true;
    plan
}

pub(super) fn graph_plan(plan: &CheckFactPlan) -> CheckFactPlan {
    CheckFactPlan {
        graph: plan.graph,
        graph_context: plan.graph_context.clone(),
        integration_runner_configs: plan.integration_runner_configs.clone(),
        ..CheckFactPlan::default()
    }
}

pub(super) fn needs_scoped_facts(plan: &CheckFactPlan) -> bool {
    plan.imports
        || plan.symbols
        || plan.react
        || plan.react_usages
        || plan.queue
        || plan.integration
        || plan.integration_runner_configs.is_some()
        || plan.dynamic_imports
        || plan.nextjs_caching
        || plan.storybook
        || plan.source
        || plan.raw_source
        || !plan.graph.is_empty()
}
