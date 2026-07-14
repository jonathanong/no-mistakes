use super::{CheckFactMap, CheckFactPlan, PlaywrightFactPlan};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

mod entrypoints;
mod finish;
mod helpers;
mod module_resolution;
mod partitions;
mod precollect;

pub(super) use entrypoints::collect_with_precollected_ts;
use finish::{finish_map, FinishMapInput};
use helpers::{
    collect_test_partition, graph_plan, has_indexable_graph_only, needs_scoped_facts, with_imports,
};
use partitions::FilePartitions;
use precollect::cached_config_graph_facts;

pub(super) fn collect_with_sources(
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>, bool),
    plan: CheckFactPlan,
    mut playwright: PlaywrightFactPlan,
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> CheckFactMap {
    module_resolution::initialize_if_missing(root, &mut playwright, &sources);
    let (files, graph_files, graph_files_complete) = file_scope;
    let precollected_ts =
        cached_config_graph_facts(&files, &graph_files, &plan, &playwright, &sources);
    collect_with_precollected_ts_and_sources(
        root,
        (files, graph_files, graph_files_complete),
        plan,
        playwright,
        precollected_ts,
        sources,
    )
}
pub(super) fn collect_with_precollected_ts_and_sources(
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>, bool),
    mut plan: CheckFactPlan,
    playwright: PlaywrightFactPlan,
    precollected_ts: crate::codebase::ts_source::facts::TsFactMap,
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> CheckFactMap {
    let (files, graph_files, graph_files_complete) = file_scope;
    assert!(
        precollected_ts.is_empty() || precollected_ts.plan().covers(plan.graph),
        "precollected graph facts must cover the staged graph plan"
    );
    let partitions = FilePartitions::new(&files, &graph_files, &playwright);
    let playwright_source_files = playwright.source_files();
    let playwright_test_files_by_project = playwright.test_files_by_project();
    let graph_only_files = super::graph_only_files(&files, &graph_files);
    let has_indexable_graph_only =
        has_indexable_graph_only(&graph_only_files, &partitions.playwright_only_sources);
    let mut ts = precollected_ts
        .into_iter()
        .map(|(path, ts)| {
            let parse_error = ts.parse_error.clone();
            let source = ts.source.as_deref().map(std::sync::Arc::<str>::from);
            (
                path,
                super::CheckFileFacts {
                    ts: ts.into(),
                    source,
                    parse_error,
                    parsed: true,
                    ..super::CheckFileFacts::default()
                },
            )
        })
        .collect::<HashMap<_, _>>();
    let uncollected_files = files
        .iter()
        .filter(|path| !ts.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    let uncollected_graph_only_files = graph_only_files
        .iter()
        .filter(|path| !ts.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    // Runner-config helpers can overlap Playwright source files. Collect import
    // facts while their cached AST is available so a later text-reachability
    // demand never has to parse those helpers again.
    let runner_plan = with_imports(plan.clone());
    let graph_fact_plan = with_imports(graph_plan(&plan));
    let ((collected, integration_runner_configs), helper_facts) =
        super::collect_prepared_runner_facts(
            root,
            &uncollected_files,
            &uncollected_graph_only_files,
            &runner_plan,
            &graph_fact_plan,
            Some(&playwright),
            std::sync::Arc::clone(&sources),
        );
    ts.extend(collected);
    ts.extend(helper_facts);
    collect_test_partition(
        root,
        &partitions.scoped_tests,
        with_imports(plan.clone()),
        &playwright,
        &sources,
        &mut ts,
    );
    collect_test_partition(
        root,
        &partitions.graph_tests,
        with_imports(graph_plan(&plan)),
        &playwright,
        &sources,
        &mut ts,
    );
    collect_test_partition(
        root,
        &partitions.playwright_only_tests,
        with_imports(CheckFactPlan::default()),
        &playwright,
        &sources,
        &mut ts,
    );
    let playwright_facts = ts
        .iter()
        .filter_map(|(path, facts)| facts.playwright.as_ref().map(|facts| (path.clone(), facts)))
        .collect::<BTreeMap<_, _>>();
    if playwright.demands_text_imports(&playwright_facts) {
        plan.graph
            .include(crate::codebase::ts_source::facts::TsFactPlan::imports());
    }
    collect_test_partition(
        root,
        &partitions.scoped_sources,
        plan.clone(),
        &playwright,
        &sources,
        &mut ts,
    );
    collect_test_partition(
        root,
        &partitions.graph_sources,
        graph_plan(&plan),
        &playwright,
        &sources,
        &mut ts,
    );
    collect_test_partition(
        root,
        &partitions.playwright_only_sources,
        graph_plan(&plan),
        &playwright,
        &sources,
        &mut ts,
    );
    if needs_scoped_facts(&plan) {
        collect_test_partition(
            root,
            &partitions.remaining_scoped,
            plan.clone(),
            &playwright,
            &sources,
            &mut ts,
        );
    }
    if !plan.graph.is_empty() {
        collect_test_partition(
            root,
            &partitions.remaining_graph,
            graph_plan(&plan),
            &playwright,
            &sources,
            &mut ts,
        );
    }
    finish_map(FinishMapInput {
        files,
        graph_files,
        graph_files_complete,
        plan,
        has_indexable_graph_only,
        files_discovered: partitions.files_discovered,
        ts,
        playwright_source_files,
        playwright_test_files_by_project,
        integration_runner_configs,
    })
}

#[cfg(test)]
mod tests;
