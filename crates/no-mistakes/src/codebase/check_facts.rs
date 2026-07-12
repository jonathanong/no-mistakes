use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::rules::nextjs_no_caching::NextjsCachingFinding;
use crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::TestFacts;
use crate::codebase::storybook::StorybookFileFacts;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_symbols::FileSymbols;
use crate::integration_tests::types::FileAnalysis as IntegrationFileAnalysis;
use crate::playwright::analysis::text_types::{AppTextTarget, PlaywrightTextLocator};
use crate::playwright::playwright_tests::TestOccurrence;
use crate::playwright::selectors::{
    AppSelector, PlaywrightHelperReference, PlaywrightSelector, SelectorRegexes,
};
use crate::react_traits::analyze::file::FileAnalysis as ReactFileAnalysis;
use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

mod file;
mod file_parse_error;
mod file_playwright;
pub(crate) use file::collect_file_facts;

#[derive(Clone, Default)]
pub struct CheckFactPlan {
    pub imports: bool,
    pub symbols: bool,
    pub react: bool,
    pub queue: bool,
    pub queue_factory_names: Vec<String>,
    pub integration: bool,
    pub dynamic_imports: bool,
    pub nextjs_caching: bool,
    pub storybook: bool,
    pub source: bool,
    pub raw_source: bool,
    pub graph: crate::codebase::ts_source::facts::TsFactPlan,
    pub graph_context: crate::codebase::ts_source::facts::TsFactContext,
}

#[derive(Clone)]
pub struct PlaywrightFactPlan {
    pub(crate) navigation_helpers: Vec<String>,
    pub(crate) selector_regexes: Arc<SelectorRegexes>,
    pub(crate) test_id_attributes_by_path: Arc<HashMap<PathBuf, Vec<String>>>,
}

#[derive(Default)]
pub struct CheckFactMap {
    pub(crate) files: Vec<PathBuf>,
    pub(crate) graph_files: Vec<PathBuf>,
    pub(crate) ts: HashMap<PathBuf, CheckFileFacts>,
    pub(crate) graph_plan: crate::codebase::ts_source::facts::TsFactPlan,
    pub stats: CheckFactStats,
    /// Memoizes the app-wide Playwright selector-occurrence scan
    /// (`collect_app_selector_occurrences`), keyed by whether HTML id
    /// attributes are included (see `TsFactLookup::get_or_compute_app_selector_occurrences`).
    /// Populated lazily — empty unless a `check` run actually triggers the
    /// scan from more than one place (e.g. the `playwright` rule and
    /// `forbidden-dependencies`'s `DepGraph` build in the same invocation).
    pub(crate) app_selector_occurrences_cache: DashMap<bool, Arc<Vec<AppSelector>>>,
    /// Memoizes `routes::collect_routes` + rewrite expansion — see
    /// `TsFactLookup::get_or_compute_playwright_routes`. Unlike the selector
    /// scan above this needs no key: every caller within one invocation wants
    /// the same routes.
    pub(crate) playwright_routes_cache: OnceLock<Arc<Vec<crate::routes::Route>>>,
    /// Memoizes `collect_app_text_targets` — see
    /// `TsFactLookup::get_or_compute_app_text_targets`.
    pub(crate) app_text_targets_cache: OnceLock<Arc<Vec<AppTextTarget>>>,
    /// Memoizes `collect_route_reachable_files` — see
    /// `TsFactLookup::get_or_compute_route_reachable_files`. The single
    /// largest cost this cache eliminates in practice.
    pub(crate) route_reachable_files_cache:
        OnceLock<Arc<crate::codebase::dependencies::graph::RouteReachableFiles>>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CheckFactStats {
    pub files_discovered: usize,
    pub files_parsed: usize,
    pub parse_errors: usize,
}

#[derive(Default)]
pub(crate) struct CheckFileFacts {
    pub ts: TsFileFacts,
    pub source: Option<String>,
    pub symbols: Option<FileSymbols>,
    pub react: Option<ReactFileAnalysis>,
    pub integration: Option<IntegrationFileAnalysis>,
    pub dynamic_imports: Option<TestFacts>,
    pub nextjs_caching: Option<Vec<NextjsCachingFinding>>,
    pub storybook: Option<StorybookFileFacts>,
    pub(crate) playwright: Option<PlaywrightTestFacts>,
    pub parse_error: Option<String>,
    pub(crate) parsed: bool,
}

pub(crate) struct PlaywrightTestFacts {
    pub(crate) urls: Vec<TestOccurrence<String>>,
    pub(crate) selectors: Vec<TestOccurrence<PlaywrightSelector>>,
    pub(crate) text_locators: Vec<TestOccurrence<PlaywrightTextLocator>>,
    pub(crate) helper_references: Vec<TestOccurrence<PlaywrightHelperReference>>,
}

impl CheckFactMap {
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub(crate) fn graph_files(&self) -> &[PathBuf] {
        match self.graph_files.is_empty() {
            true => &self.files,
            false => &self.graph_files,
        }
    }

    pub(crate) fn graph_plan(&self) -> crate::codebase::ts_source::facts::TsFactPlan {
        self.graph_plan
    }
}

pub fn collect_check_facts(root: &Path, files: Vec<PathBuf>, plan: CheckFactPlan) -> CheckFactMap {
    collect_check_facts_with_playwright(root, files, plan, None)
}

pub fn collect_check_facts_with_graph_files_and_playwright(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
) -> CheckFactMap {
    collect_check_facts_inner(root, files, graph_files, plan, playwright)
}

pub fn collect_check_facts_with_playwright(
    root: &Path,
    files: Vec<PathBuf>,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
) -> CheckFactMap {
    collect_check_facts_inner(root, files, Vec::new(), plan, playwright)
}

fn collect_check_facts_inner(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
) -> CheckFactMap {
    let graph_only_files = graph_only_files(&files, &graph_files);
    let stats = CheckFactStats {
        files_discovered: files.len() + graph_only_files.len(),
        ..CheckFactStats::default()
    };
    let playwright = playwright.as_ref();
    let mut ts = collect_fact_map(root, &files, &plan, playwright);
    if !graph_only_files.is_empty() {
        let graph_plan = CheckFactPlan {
            graph: plan.graph,
            graph_context: plan.graph_context.clone(),
            ..CheckFactPlan::default()
        };
        ts.extend(collect_fact_map(root, &graph_only_files, &graph_plan, None));
    }
    let mut files_parsed = 0;
    let mut parse_errors = 0;
    for facts in ts.values() {
        if facts.parsed {
            files_parsed += 1;
        }
        if facts.parse_error.is_some() {
            parse_errors += 1;
        }
    }
    CheckFactMap {
        files,
        graph_files,
        ts,
        graph_plan: plan.graph,
        stats: CheckFactStats {
            files_parsed,
            parse_errors,
            ..stats
        },
        app_selector_occurrences_cache: DashMap::new(),
        playwright_routes_cache: OnceLock::new(),
        app_text_targets_cache: OnceLock::new(),
        route_reachable_files_cache: OnceLock::new(),
    }
}

fn collect_fact_map(
    root: &Path,
    files: &[PathBuf],
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
) -> HashMap<PathBuf, CheckFileFacts> {
    files
        .par_iter()
        .filter(|path| is_indexable(path) || (plan.storybook && is_mdx_file(path)))
        .filter_map(|path| {
            collect_file_facts(root, path, plan, playwright).map(|facts| (path.clone(), facts))
        })
        .collect()
}

fn graph_only_files(files: &[PathBuf], graph_files: &[PathBuf]) -> Vec<PathBuf> {
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

fn is_mdx_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("mdx")
}

#[cfg(test)]
mod tests;
