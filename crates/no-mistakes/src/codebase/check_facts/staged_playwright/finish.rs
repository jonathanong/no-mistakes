use super::super::{
    CheckFactMap, CheckFactPlan, CheckFactStats, CheckFileFacts, PlaywrightTestFilesByProject,
    RunnerConfigFacts,
};
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub(super) struct FinishMapInput {
    pub(super) files: Vec<PathBuf>,
    pub(super) graph_files: Vec<PathBuf>,
    pub(super) graph_files_complete: bool,
    pub(super) plan: CheckFactPlan,
    pub(super) has_indexable_graph_only: bool,
    pub(super) files_discovered: usize,
    pub(super) ts: HashMap<PathBuf, CheckFileFacts>,
    pub(super) playwright_source_files: Arc<Vec<PathBuf>>,
    pub(super) playwright_test_files_by_project: PlaywrightTestFilesByProject,
    pub(super) integration_runner_configs: RunnerConfigFacts,
}

pub(super) fn finish_map(input: FinishMapInput) -> CheckFactMap {
    let FinishMapInput {
        files,
        mut graph_files,
        graph_files_complete,
        plan,
        has_indexable_graph_only,
        files_discovered,
        ts,
        playwright_source_files,
        playwright_test_files_by_project,
        mut integration_runner_configs,
    } = input;
    graph_files.extend(playwright_source_files.iter().cloned());
    graph_files.sort();
    graph_files.dedup();
    let graph_files_complete = graph_files_complete || !playwright_source_files.is_empty();
    let files_parsed = ts.values().filter(|facts| facts.parsed).count();
    let parse_errors = ts
        .values()
        .filter(|facts| facts.parse_error.is_some())
        .count();
    integration_runner_configs.extend(super::super::runner_config_facts(&ts));
    let (app_selector_occurrences_cache, app_text_targets_cache) =
        super::super::playwright_aggregate_facts(&ts);
    CheckFactMap {
        files,
        graph_files,
        graph_files_complete,
        ts,
        graph_plan: if has_indexable_graph_only {
            plan.graph
        } else {
            plan.collected_ts_plan()
        },
        integration_runner_configs,
        playwright_source_files,
        playwright_test_files_by_project,
        stats: CheckFactStats {
            files_discovered,
            files_parsed,
            parse_errors,
        },
        app_selector_occurrences_cache,
        playwright_routes_cache: DashMap::new(),
        app_text_targets_cache,
        route_reachable_files_cache: DashMap::new(),
    }
}
