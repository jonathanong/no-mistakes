use super::{CheckFactMap, CheckFactPlan, CheckFactStats, PlaywrightFactPlan};
use crate::codebase::dependencies::extract::is_indexable;
use dashmap::DashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) use super::collect_helpers::graph_only_files;
pub(super) use super::collect_helpers::request_sources;
use super::collect_helpers::uncollected_files;
pub(super) use super::collect_helpers::{
    collect_fact_map_sequential_with_sources, collect_fact_map_with_sources,
};

#[doc(hidden)]
pub fn collect_check_facts_with_graph_files_playwright_and_sources(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
    sources: Arc<crate::codebase::ts_source::SourceStore>,
) -> CheckFactMap {
    collect_check_facts_with_graph_files_and_playwright_impl(
        root,
        files,
        graph_files,
        plan,
        playwright,
        Some(sources),
    )
}

pub(super) fn collect_check_facts_with_graph_files_and_playwright_impl(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    mut plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
    sources: Option<Arc<crate::codebase::ts_source::SourceStore>>,
) -> CheckFactMap {
    if plan.graph_context.visible_files.is_none() {
        let mut visible_files = files.clone();
        visible_files.extend(graph_files.iter().cloned());
        if let Some(playwright) = &playwright {
            visible_files.extend(playwright.source_files().iter().cloned());
        }
        plan.graph_context.set_visible_files(visible_files);
    }
    let sources = sources
        .unwrap_or_else(|| request_sources(&files, &graph_files, &plan, playwright.as_ref()));
    if let Some(playwright) = playwright {
        return super::staged_playwright::collect_with_sources(
            root,
            (files, graph_files, true),
            plan,
            playwright,
            sources,
        );
    }
    collect_check_facts_inner(root, files, graph_files, true, plan, playwright, sources)
}

pub(crate) fn collect_check_facts_with_precollected_graph_facts(
    root: &Path,
    graph_files: Vec<PathBuf>,
    mut plan: CheckFactPlan,
    playwright: PlaywrightFactPlan,
    precollected_ts: crate::codebase::ts_source::facts::TsFactMap,
) -> CheckFactMap {
    if plan.graph_context.visible_files.is_none() {
        let mut visible_files = graph_files.clone();
        visible_files.extend(playwright.source_files().iter().cloned());
        plan.graph_context.set_visible_files(visible_files);
    }
    super::staged_playwright::collect_with_precollected_ts(
        root,
        Vec::new(),
        graph_files,
        true,
        plan,
        playwright,
        precollected_ts,
    )
}

pub fn collect_check_facts_with_playwright(
    root: &Path,
    files: Vec<PathBuf>,
    mut plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
) -> CheckFactMap {
    if plan.graph_context.visible_files.is_none() {
        let mut visible_files = files.clone();
        if let Some(playwright) = &playwright {
            visible_files.extend(playwright.source_files().iter().cloned());
        }
        plan.graph_context.set_visible_files(visible_files);
    }
    let sources = request_sources(&files, &[], &plan, playwright.as_ref());
    if let Some(playwright) = playwright {
        return super::staged_playwright::collect_with_sources(
            root,
            (files, Vec::new(), false),
            plan,
            playwright,
            sources,
        );
    }
    collect_check_facts_inner(root, files, Vec::new(), false, plan, playwright, sources)
}

fn collect_check_facts_inner(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    graph_files_complete: bool,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
    sources: Arc<crate::codebase::ts_source::SourceStore>,
) -> CheckFactMap {
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
    let ((mut ts, mut integration_runner_configs), helper_facts) =
        super::collect_prepared_runner_facts(
            root,
            &files,
            &graph_only_files,
            &plan,
            &graph_plan,
            playwright,
            Arc::clone(&sources),
        );
    let helper_paths = helper_facts.keys().cloned().collect::<HashSet<_>>();
    ts.extend(helper_facts);
    let remaining_files = uncollected_files(&files, &ts, &helper_paths);
    let remaining_graph_files = uncollected_files(&graph_only_files, &ts, &helper_paths);
    ts.extend(collect_fact_map_with_sources(
        root,
        &remaining_files,
        &plan,
        playwright,
        &sources,
    ));
    ts.extend(collect_fact_map_with_sources(
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
