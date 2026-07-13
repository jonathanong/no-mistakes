use super::{CheckFactMap, CheckFactPlan, PlaywrightFactPlan};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

mod finish;
mod helpers;
mod partitions;
mod precollect;

use finish::{finish_map, FinishMapInput};
use helpers::{collect_test_partition, graph_plan, needs_scoped_facts, with_imports};
use partitions::FilePartitions;
use precollect::cached_config_graph_facts;

pub(super) fn collect(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    graph_files_complete: bool,
    plan: CheckFactPlan,
    playwright: PlaywrightFactPlan,
) -> CheckFactMap {
    let precollected_ts = cached_config_graph_facts(&files, &graph_files, &plan, &playwright);
    collect_with_precollected_ts(
        root,
        files,
        graph_files,
        graph_files_complete,
        plan,
        playwright,
        precollected_ts,
    )
}

pub(super) fn collect_with_precollected_ts(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    graph_files_complete: bool,
    mut plan: CheckFactPlan,
    playwright: PlaywrightFactPlan,
    precollected_ts: crate::codebase::ts_source::facts::TsFactMap,
) -> CheckFactMap {
    assert!(
        precollected_ts.is_empty() || precollected_ts.plan().covers(plan.graph),
        "precollected graph facts must cover the staged graph plan"
    );
    let partitions = FilePartitions::new(&files, &graph_files, &playwright);
    let playwright_source_files = playwright.source_files();
    let playwright_test_files_by_project = playwright.test_files_by_project();
    let graph_only_files = super::graph_only_files(&files, &graph_files);
    let has_indexable_graph_only = graph_only_files
        .iter()
        .chain(partitions.playwright_only_sources.iter())
        .any(|path| crate::codebase::dependencies::extract::is_indexable(path));
    let mut ts = precollected_ts
        .into_iter()
        .map(|(path, ts)| {
            let parse_error = ts.parse_error.clone();
            let source = ts.source.clone();
            (
                path,
                super::CheckFileFacts {
                    ts,
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
        );
    ts.extend(collected);
    ts.extend(helper_facts);
    collect_test_partition(
        root,
        &partitions.scoped_tests,
        with_imports(plan.clone()),
        &playwright,
        &mut ts,
    );
    collect_test_partition(
        root,
        &partitions.graph_tests,
        with_imports(graph_plan(&plan)),
        &playwright,
        &mut ts,
    );
    collect_test_partition(
        root,
        &partitions.playwright_only_tests,
        with_imports(CheckFactPlan::default()),
        &playwright,
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
        &mut ts,
    );
    collect_test_partition(
        root,
        &partitions.graph_sources,
        graph_plan(&plan),
        &playwright,
        &mut ts,
    );
    collect_test_partition(
        root,
        &partitions.playwright_only_sources,
        graph_plan(&plan),
        &playwright,
        &mut ts,
    );
    if needs_scoped_facts(&plan) {
        collect_test_partition(
            root,
            &partitions.remaining_scoped,
            plan.clone(),
            &playwright,
            &mut ts,
        );
    }
    if !plan.graph.is_empty() {
        collect_test_partition(
            root,
            &partitions.remaining_graph,
            graph_plan(&plan),
            &playwright,
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
