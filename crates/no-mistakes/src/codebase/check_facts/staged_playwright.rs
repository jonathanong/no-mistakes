use super::{CheckFactMap, CheckFactPlan, PlaywrightFactPlan};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod entrypoints;
mod finish;
mod helpers;
mod module_resolution;
mod partition_collection;
mod partitions;
mod precollect;

pub(super) use entrypoints::collect_with_precollected_ts;
use finish::{finish_map, FinishMapInput};
use helpers::{graph_plan, has_indexable_graph_only, with_imports};
use partition_collection::collect_partitions;
use partitions::FilePartitions;
use precollect::cached_config_file_facts;

pub(super) struct PrecollectedFacts {
    ts: crate::codebase::ts_source::facts::TsFactMap,
    files: HashMap<PathBuf, super::CheckFileFacts>,
}

pub(super) fn collect_with_sources_and_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>, bool),
    plan: CheckFactPlan,
    mut playwright: PlaywrightFactPlan,
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
    precollected: HashMap<PathBuf, super::CheckFileFacts>,
) -> CheckFactMap {
    module_resolution::initialize_if_missing(root, &mut playwright, &sources);
    let config_facts = cached_config_file_facts(
        session,
        root,
        &file_scope.0,
        &file_scope.1,
        &plan,
        &playwright,
        &sources,
    );
    let mut precollected = precollected;
    precollected.extend(config_facts);
    collect_with_precollected_ts_sources_and_session(
        session,
        root,
        file_scope,
        plan,
        playwright,
        PrecollectedFacts {
            ts: crate::codebase::ts_source::facts::TsFactMap::new(),
            files: precollected,
        },
        sources,
    )
}

pub(super) fn collect_with_precollected_ts_sources_and_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>, bool),
    mut plan: CheckFactPlan,
    playwright: PlaywrightFactPlan,
    precollected: PrecollectedFacts,
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> CheckFactMap {
    let PrecollectedFacts {
        ts: precollected_ts,
        files: precollected,
    } = precollected;
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
    ts.extend(precollected);
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
    // facts while their cached AST is available so later demands never reparse.
    let runner_plan = with_imports(plan.clone());
    let graph_fact_plan = with_imports(graph_plan(&plan));
    let ((collected, integration_runner_configs), helper_facts) =
        super::collect_prepared_runner_facts(
            session,
            root,
            (&uncollected_files, &uncollected_graph_only_files),
            &runner_plan,
            &graph_fact_plan,
            Some(&playwright),
            std::sync::Arc::clone(&sources),
        );
    ts.extend(collected);
    ts.extend(helper_facts);
    collect_partitions(
        session,
        root,
        &partitions,
        &mut plan,
        &playwright,
        &sources,
        &mut ts,
    );
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
