use super::{CheckFactMap, CheckFactPlan, CheckFactStats, CheckFileFacts, PlaywrightFactPlan};
use crate::codebase::dependencies::extract::is_indexable;
use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn collect_check_facts(root: &Path, files: Vec<PathBuf>, plan: CheckFactPlan) -> CheckFactMap {
    collect_check_facts_with_playwright(root, files, plan, None)
}

pub fn collect_check_facts_with_graph_files_and_playwright(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    mut plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
) -> CheckFactMap {
    if plan.graph_context.visible_files.is_none() {
        let mut visible_files = files.clone();
        visible_files.extend(graph_files.iter().cloned());
        if let Some(playwright) = &playwright {
            visible_files.extend(playwright.source_files().iter().cloned());
        }
        plan.graph_context.set_visible_files(visible_files);
    }
    if let Some(playwright) = playwright {
        return super::staged_playwright::collect(root, files, graph_files, true, plan, playwright);
    }
    collect_check_facts_inner(root, files, graph_files, true, plan, playwright)
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
    if let Some(playwright) = playwright {
        return super::staged_playwright::collect(root, files, Vec::new(), false, plan, playwright);
    }
    collect_check_facts_inner(root, files, Vec::new(), false, plan, playwright)
}

fn collect_check_facts_inner(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    graph_files_complete: bool,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
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
        );
    let helper_paths = helper_facts.keys().cloned().collect::<HashSet<_>>();
    ts.extend(helper_facts);
    let remaining_files = uncollected_files(&files, &ts, &helper_paths);
    let remaining_graph_files = uncollected_files(&graph_only_files, &ts, &helper_paths);
    ts.extend(collect_fact_map(root, &remaining_files, &plan, playwright));
    ts.extend(collect_fact_map(
        root,
        &remaining_graph_files,
        &graph_plan,
        playwright,
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
        ts,
        graph_plan: collected_ts_plan,
        integration_runner_configs,
        playwright_source_files: Arc::new(Vec::new()),
        playwright_test_files_by_project: Arc::new(Vec::new()),
        stats: CheckFactStats {
            files_parsed,
            parse_errors,
            ..stats
        },
        app_selector_occurrences_cache: DashMap::new(),
        playwright_routes_cache: DashMap::new(),
        app_text_targets_cache: DashMap::new(),
        route_reachable_files_cache: DashMap::new(),
    }
}

fn uncollected_files(
    files: &[PathBuf],
    facts: &HashMap<PathBuf, CheckFileFacts>,
    helper_paths: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    files
        .iter()
        .filter(|path| {
            !facts.contains_key(*path)
                && !helper_paths.contains(&crate::codebase::ts_resolver::normalize_path(path))
        })
        .cloned()
        .collect()
}

pub(crate) fn collect_fact_map(
    root: &Path,
    files: &[PathBuf],
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
) -> HashMap<PathBuf, CheckFileFacts> {
    files
        .par_iter()
        .filter(|path| is_indexable(path) || (plan.storybook && super::is_mdx_file(path)))
        .filter_map(|path| {
            super::collect_file_facts(root, path, plan, playwright)
                .map(|facts| (path.clone(), facts))
        })
        .collect()
}

pub(super) fn collect_fact_map_sequential(
    root: &Path,
    files: &[PathBuf],
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
) -> HashMap<PathBuf, CheckFileFacts> {
    files
        .iter()
        .filter(|path| is_indexable(path) || (plan.storybook && super::is_mdx_file(path)))
        .filter_map(|path| {
            super::collect_file_facts(root, path, plan, playwright)
                .map(|facts| (path.clone(), facts))
        })
        .collect()
}

pub(crate) fn graph_only_files(files: &[PathBuf], graph_files: &[PathBuf]) -> Vec<PathBuf> {
    if graph_files.is_empty() {
        return Vec::new();
    }
    let scoped: BTreeSet<&PathBuf> = files.iter().collect();
    graph_files
        .iter()
        .filter(|path| !scoped.contains(path))
        .cloned()
        .collect()
}
