use super::{CheckFactStats, PlaywrightSettingsKey, PlaywrightTestFacts};
use crate::codebase::rules::nextjs_no_caching::NextjsCachingFinding;
use crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::TestFacts;
use crate::codebase::storybook::StorybookFileFacts;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_symbols::FileSymbols;
use crate::integration_tests::types::FileAnalysis as IntegrationFileAnalysis;
use crate::playwright::analysis::text_types::AppTextTarget;
use crate::playwright::selectors::{AppSelector, StaticExportValues};
use crate::react_traits::analyze::file::FileAnalysis as ReactFileAnalysis;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub(crate) type PlaywrightTestFilesByProject = Arc<
    Vec<(
        Option<String>,
        Arc<Vec<crate::playwright::analysis::context::DiscoveredTestFile>>,
    )>,
>;
type AppSelectorOccurrencesCache =
    Arc<DashMap<(PlaywrightSettingsKey, bool), Result<Arc<Vec<AppSelector>>, String>>>;
type PlaywrightRoutesCache = Arc<DashMap<PlaywrightSettingsKey, Arc<Vec<crate::routes::Route>>>>;
type AppTextTargetsCache =
    Arc<DashMap<PlaywrightSettingsKey, Result<Arc<Vec<AppTextTarget>>, String>>>;
type RouteReachableFilesCache = Arc<
    DashMap<
        PlaywrightSettingsKey,
        Result<Arc<crate::codebase::dependencies::graph::RouteReachableFiles>, String>,
    >,
>;

#[derive(Default)]
pub struct CheckFactMap {
    pub(crate) files: Vec<PathBuf>,
    pub(crate) graph_files: Vec<PathBuf>,
    pub(crate) graph_files_complete: bool,
    pub(crate) ts: HashMap<PathBuf, Arc<CheckFileFacts>>,
    pub(crate) graph_plan: crate::codebase::ts_source::facts::TsFactPlan,
    pub(crate) integration_runner_configs: std::collections::BTreeMap<
        PathBuf,
        crate::integration_tests::runner_config::RunnerConfigFileFacts,
    >,
    pub(crate) playwright_source_files: Arc<Vec<PathBuf>>,
    pub(crate) playwright_test_files_by_project: PlaywrightTestFilesByProject,
    pub stats: CheckFactStats,
    pub(crate) app_selector_occurrences_cache: AppSelectorOccurrencesCache,
    pub(crate) playwright_routes_cache: PlaywrightRoutesCache,
    pub(crate) app_text_targets_cache: AppTextTargetsCache,
    pub(crate) route_reachable_files_cache: RouteReachableFilesCache,
}

#[derive(Default)]
pub(crate) struct CheckFileFacts {
    pub ts: Arc<TsFileFacts>,
    pub source: Option<Arc<str>>,
    pub symbols: Option<Arc<FileSymbols>>,
    pub(crate) legacy_symbols: Option<Arc<FileSymbols>>,
    pub react: Option<Arc<ReactFileAnalysis>>,
    pub(crate) react_usages: Option<crate::react_traits::pipeline::usages::UsageFileFacts>,
    pub integration: Option<IntegrationFileAnalysis>,
    pub(crate) integration_runner_config:
        Option<crate::integration_tests::runner_config::RunnerConfigFileFacts>,
    pub dynamic_imports: Option<TestFacts>,
    pub nextjs_caching: Option<Vec<NextjsCachingFinding>>,
    pub storybook: Option<StorybookFileFacts>,
    pub(crate) server_route_client_boundary:
        Option<crate::codebase::rules::server_route_client_boundary::FileFacts>,
    pub(crate) playwright: Option<PlaywrightTestFacts>,
    pub(crate) playwright_fetch: Option<crate::fetch::file_facts::ParsedFileFacts>,
    pub(crate) playwright_app_selectors: HashMap<(PlaywrightSettingsKey, bool), Vec<AppSelector>>,
    pub(crate) playwright_app_text_targets: HashMap<PlaywrightSettingsKey, Vec<AppTextTarget>>,
    pub(crate) playwright_static_exports: Option<StaticExportValues>,
    pub parse_error: Option<String>,
    pub(crate) legacy_symbol_parse_error: Option<String>,
    pub(crate) parsed: bool,
}

impl CheckFactMap {
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub(crate) fn graph_file_universe(&self) -> &[PathBuf] {
        if self.graph_files_complete {
            self.graph_files.as_slice()
        } else {
            self.files.as_slice()
        }
    }

    pub(crate) fn graph_plan(&self) -> crate::codebase::ts_source::facts::TsFactPlan {
        self.graph_plan
    }

    pub(crate) fn graph_view_with_supplemental(&self, supplemental: &Self) -> Self {
        let mut graph_files = self.graph_files.clone();
        graph_files.extend(supplemental.ts.keys().cloned());
        graph_files.sort();
        graph_files.dedup();
        self.view_with_supplemental(supplemental, graph_files)
    }

    fn view_with_supplemental(&self, supplemental: &Self, graph_files: Vec<PathBuf>) -> Self {
        let mut ts = self.ts.clone();
        ts.extend(
            supplemental
                .ts
                .iter()
                .map(|(path, facts)| (path.clone(), Arc::clone(facts))),
        );
        let mut graph_plan = self.graph_plan;
        graph_plan.include(supplemental.graph_plan);
        Self {
            files: self.files.clone(),
            graph_files,
            graph_files_complete: self.graph_files_complete,
            ts,
            graph_plan,
            integration_runner_configs: self.integration_runner_configs.clone(),
            playwright_source_files: Arc::clone(&self.playwright_source_files),
            playwright_test_files_by_project: Arc::clone(&self.playwright_test_files_by_project),
            app_selector_occurrences_cache: Arc::clone(&self.app_selector_occurrences_cache),
            playwright_routes_cache: Arc::clone(&self.playwright_routes_cache),
            app_text_targets_cache: Arc::clone(&self.app_text_targets_cache),
            route_reachable_files_cache: Arc::clone(&self.route_reachable_files_cache),
            stats: self.stats,
        }
    }
}
