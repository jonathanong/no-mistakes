use super::super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn collect_test_partition(
    root: &Path,
    files: &[PathBuf],
    plan: CheckFactPlan,
    playwright: &PlaywrightFactPlan,
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &mut HashMap<PathBuf, CheckFileFacts>,
) {
    let files = files
        .iter()
        .filter(|path| !facts.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    facts.extend(super::super::collect::collect_fact_map_with_sources(
        root,
        &files,
        &plan,
        Some(playwright),
        sources,
    ));
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

pub(super) fn has_indexable_graph_only(
    graph_only_files: &[PathBuf],
    playwright_only_sources: &[PathBuf],
) -> bool {
    graph_only_files
        .iter()
        .chain(playwright_only_sources)
        .any(|path| crate::codebase::dependencies::extract::is_indexable(path))
}
