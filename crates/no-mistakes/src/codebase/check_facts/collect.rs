use super::{CheckFactMap, CheckFactPlan, CheckFactStats, CheckFileFacts, PlaywrightFactPlan};
use crate::codebase::dependencies::extract::is_indexable;
use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod entrypoints;

pub(crate) use super::collect_helpers::graph_only_files;
pub(super) use super::collect_helpers::request_sources;
use super::collect_helpers::uncollected_files;
pub(super) use super::collect_helpers::{
    collect_fact_map_sequential_with_sources, collect_fact_map_with_sources,
};
pub(crate) use entrypoints::collect_check_facts_with_precollected_file_facts;
pub(crate) use entrypoints::collect_check_facts_with_precollected_graph_facts;
pub use entrypoints::{
    collect_check_facts, collect_check_facts_with_graph_files_and_playwright,
    collect_check_facts_with_graph_files_playwright_and_session,
    collect_check_facts_with_graph_files_playwright_and_sources,
    collect_check_facts_with_graph_files_playwright_sources_and_session,
    collect_check_facts_with_playwright, collect_check_facts_with_playwright_and_session,
};

fn collect_check_facts_inner(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>, bool),
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
    sources: Arc<crate::codebase::ts_source::SourceStore>,
    mut ts: HashMap<PathBuf, CheckFileFacts>,
) -> CheckFactMap {
    let (files, graph_files, graph_files_complete) = file_scope;
    let graph_only_files = graph_only_files(&files, &graph_files);
    let collected_ts_plan = if graph_only_files.iter().any(|path| is_indexable(path)) {
        plan.graph
    } else {
        plan.collected_ts_plan()
    };
    let stats = CheckFactStats {
        files_discovered: files.len() + graph_only_files.len(),
        ..CheckFactStats::default()
    };
    let playwright = playwright.as_ref();
    let graph_plan = CheckFactPlan {
        graph: plan.graph,
        graph_context: plan.graph_context.clone(),
        integration_runner_configs: plan.integration_runner_configs.clone(),
        ..CheckFactPlan::default()
    };
    let ((collected, mut integration_runner_configs), helper_facts) =
        super::collect_prepared_runner_facts(
            session,
            root,
            (&files, &graph_only_files),
            &plan,
            &graph_plan,
            playwright,
            Arc::clone(&sources),
        );
    ts.extend(collected);
    let helper_paths = helper_facts.keys().cloned().collect::<HashSet<_>>();
    ts.extend(helper_facts);
    let remaining_files = uncollected_files(&files, &ts, &helper_paths);
    let remaining_graph_files = uncollected_files(&graph_only_files, &ts, &helper_paths);
    ts.extend(collect_fact_map_with_sources(
        session,
        root,
        &remaining_files,
        &plan,
        playwright,
        &sources,
    ));
    ts.extend(collect_fact_map_with_sources(
        session,
        root,
        &remaining_graph_files,
        &graph_plan,
        playwright,
        &sources,
    ));
    integration_runner_configs.extend(super::runner_config_facts(&ts));
    let files_parsed = ts.values().filter(|facts| facts.parsed).count();
    let parse_errors = ts
        .values()
        .filter(|facts| facts.parse_error.is_some())
        .count();
    CheckFactMap {
        files,
        graph_files,
        graph_files_complete,
        ts: ts
            .into_iter()
            .map(|(path, facts)| (path, Arc::new(facts)))
            .collect(),
        graph_plan: collected_ts_plan,
        integration_runner_configs,
        playwright_source_files: Arc::new(Vec::new()),
        playwright_test_files_by_project: Arc::new(Vec::new()),
        stats: CheckFactStats {
            files_parsed,
            parse_errors,
            ..stats
        },
        app_selector_occurrences_cache: Arc::new(DashMap::new()),
        playwright_routes_cache: Arc::new(DashMap::new()),
        app_text_targets_cache: Arc::new(DashMap::new()),
        route_reachable_files_cache: Arc::new(DashMap::new()),
    }
}
