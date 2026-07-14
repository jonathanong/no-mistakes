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
    DashMap<(PlaywrightSettingsKey, bool), Result<Arc<Vec<AppSelector>>, String>>;
type PlaywrightRoutesCache = DashMap<PlaywrightSettingsKey, Arc<Vec<crate::routes::Route>>>;
type AppTextTargetsCache = DashMap<PlaywrightSettingsKey, Result<Arc<Vec<AppTextTarget>>, String>>;
type RouteReachableFilesCache = DashMap<
    PlaywrightSettingsKey,
    Result<Arc<crate::codebase::dependencies::graph::RouteReachableFiles>, String>,
>;

#[derive(Default)]
pub struct CheckFactMap {
    pub(crate) files: Vec<PathBuf>,
    pub(crate) graph_files: Vec<PathBuf>,
    pub(crate) graph_files_complete: bool,
    pub(crate) ts: HashMap<PathBuf, CheckFileFacts>,
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
    pub ts: TsFileFacts,
    pub source: Option<String>,
    pub symbols: Option<FileSymbols>,
    pub react: Option<ReactFileAnalysis>,
    pub(crate) react_usages: Option<crate::react_traits::pipeline::usages::UsageFileFacts>,
    pub integration: Option<IntegrationFileAnalysis>,
    pub(crate) integration_runner_config:
        Option<crate::integration_tests::runner_config::RunnerConfigFileFacts>,
    pub dynamic_imports: Option<TestFacts>,
    pub nextjs_caching: Option<Vec<NextjsCachingFinding>>,
    pub storybook: Option<StorybookFileFacts>,
    pub(crate) playwright: Option<PlaywrightTestFacts>,
    pub(crate) playwright_fetch: Option<crate::fetch::file_facts::ParsedFileFacts>,
    pub(crate) playwright_app_selectors: HashMap<(PlaywrightSettingsKey, bool), Vec<AppSelector>>,
    pub(crate) playwright_app_text_targets: HashMap<PlaywrightSettingsKey, Vec<AppTextTarget>>,
    pub(crate) playwright_static_exports: Option<StaticExportValues>,
    pub parse_error: Option<String>,
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
}
