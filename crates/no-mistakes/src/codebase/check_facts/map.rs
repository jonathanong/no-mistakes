mod access;
#[path = "map_views.rs"]
mod views;

mod fragments;
mod postgres_catalog;
mod postgres_readable;
mod sql_verdicts;
use super::{CheckFactStats, PlaywrightSettingsKey, PlaywrightTestFacts};
use crate::codebase::rules::nextjs_no_caching::NextjsCachingFinding;
use crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::TestFacts;
use crate::codebase::storybook::StorybookFileFacts;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::FileIdMap;
use crate::codebase::ts_symbols::FileSymbols;
use crate::integration_tests::types::FileAnalysis as IntegrationFileAnalysis;
use crate::playwright::analysis::text_types::AppTextTarget;
use crate::playwright::selectors::{AppSelector, StaticExportValues};
use crate::react_traits::analyze::file::FileAnalysis as ReactFileAnalysis;
use dashmap::DashMap;
pub use sql_verdicts::{SqlApplicationFacts, SqlFactContext};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
    pub(crate) sql_application: Option<crate::codebase::postgres::SqlRuleApplication>,
    pub(crate) sql_verdicts:
        Option<Arc<Mutex<Vec<crate::codebase::postgres::SqlObligationVerdict>>>>,
    pub(crate) companion_import_plans: Arc<
        Vec<Arc<crate::codebase::rules::required_companion_imports::PreparedCompanionImportPlan>>,
    >,
    pub(crate) postgres: crate::codebase::postgres::prepared::PreparedPostgresFacts,
    pub(crate) integration_route_links: crate::playwright::integration_routes::PreparedLinks,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) graph_files: Vec<PathBuf>,
    pub(crate) graph_files_complete: bool,
    pub(crate) ts: FileIdMap<Arc<CheckFileFacts>>,
    pub(crate) graph_plan: crate::codebase::ts_source::facts::TsFactPlan,
    /// Per-file call-site coverage, separate from graph-wide fact coverage.
    pub(crate) call_site_paths: Arc<Vec<PathBuf>>,
    pub(crate) integration_runner_configs: std::collections::BTreeMap<
        PathBuf,
        crate::integration_tests::runner_config::RunnerConfigFileFacts,
    >,
    pub(crate) playwright_source_files: Arc<Vec<PathBuf>>,
    pub(crate) playwright_test_files_by_project: PlaywrightTestFilesByProject,
    pub stats: CheckFactStats,
    /// Deduped SQL-looking member `.query` calls that were not proven executors.
    /// Printed by the CLI; it is not a finding and does not fail the run.
    pub(crate) unproven_member_query_sql: u32,
    pub(crate) app_selector_occurrences_cache: AppSelectorOccurrencesCache,
    pub(crate) playwright_routes_cache: PlaywrightRoutesCache,
    pub(crate) app_text_targets_cache: AppTextTargetsCache,
    pub(crate) route_reachable_files_cache: RouteReachableFilesCache,
    pub(crate) postgres_schema_catalogs: std::collections::BTreeMap<
        String,
        Result<Arc<crate::codebase::postgres::SchemaCatalog>, Arc<str>>,
    >,
}

impl CheckFactMap {
    /// SQL-looking member `.query` calls that were not proven to be executors.
    #[doc(hidden)]
    pub fn unproven_member_query_sql(&self) -> u32 {
        self.unproven_member_query_sql
    }
}

#[derive(Default)]
pub(crate) struct CheckFileFacts {
    pub ts: Arc<TsFileFacts>,
    pub(crate) production_imports:
        Option<Arc<Vec<crate::codebase::dependencies::extract::ExtractedImport>>>,
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
    pub(crate) embedded_sql: Vec<(
        crate::codebase::postgres::EmbeddedSqlOptions,
        crate::codebase::postgres::EmbeddedSqlFileFacts,
    )>,
    pub(crate) query_annotation_heads:
        Option<Arc<crate::codebase::postgres::query_annotation::heads::AnnotationHeads>>,
    pub(crate) query_annotation: Vec<(
        crate::codebase::postgres::EmbeddedSqlOptions,
        crate::codebase::postgres::query_annotation::QueryAnnotationFileFacts,
    )>,
    pub(crate) server_route_client_boundary:
        Option<crate::codebase::rules::server_route_client_boundary::FileFacts>,
    pub(crate) playwright: Option<PlaywrightTestFacts>,
    pub(crate) playwright_fetch: Option<crate::fetch::file_facts::ParsedFileFacts>,
    pub(crate) playwright_app_selectors: HashMap<(PlaywrightSettingsKey, bool), Vec<AppSelector>>,
    pub(crate) playwright_app_text_targets: HashMap<PlaywrightSettingsKey, Vec<AppTextTarget>>,
    pub(crate) playwright_static_exports: Option<StaticExportValues>,
    pub(crate) integration_route_occurrences:
        Vec<crate::playwright::integration_routes::RouteOccurrence>,
    pub parse_error: Option<String>,
    pub(crate) legacy_symbol_parse_error: Option<String>,
    pub(crate) parsed: bool,
}
