mod access;
#[path = "map_views.rs"]
mod views;

use super::{CheckFactStats, PlaywrightSettingsKey, PlaywrightTestFacts};
use crate::codebase::playwright_facts::text::AppTextTarget;
use crate::codebase::playwright_facts::{selectors::AppSelector, StaticExportValues};
use crate::codebase::rules::nextjs_no_caching::NextjsCachingFinding;
use crate::codebase::storybook::StorybookFileFacts;
use crate::codebase::testing_analysis::dynamic_imports::TestFacts;
use crate::codebase::ts_source::facts::react_analysis::{
    component::FileAnalysis as ReactFileAnalysis, usages::UsageFileFacts,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::FileIdMap;
use crate::codebase::ts_symbols::FileSymbols;
use crate::integration_tests::types::FileAnalysis as IntegrationFileAnalysis;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

pub(crate) type PlaywrightTestFilesByProject = Arc<
    Vec<(
        Option<String>,
        Arc<Vec<crate::codebase::playwright_facts::DiscoveredTestFile>>,
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
    pub postgres_executor_warning_scans:
        Arc<Vec<crate::codebase::postgres_analysis::executor_warning_scan::ExecutorWarningScan>>,
    pub sql_application: Option<crate::codebase::postgres_analysis::verdict::SqlRuleApplication>,
    pub sql_verdicts:
        Option<Arc<Mutex<Vec<crate::codebase::postgres_analysis::verdict::SqlObligationVerdict>>>>,
    pub sql_advisories: Arc<
        OnceLock<Mutex<Vec<crate::codebase::postgres_analysis::verdict::SqlObligationVerdict>>>,
    >,
    pub accepted_statement_bounds: Arc<Mutex<Vec<super::accepted_bounds::AcceptedStatementBound>>>,
    pub companion_import_plans: Arc<
        Vec<Arc<crate::codebase::rules::required_companion_imports::PreparedCompanionImportPlan>>,
    >,
    pub postgres_payload: crate::codebase::postgres_analysis::MapPayload,
    pub(crate) integration_route_links:
        crate::codebase::playwright_facts::integration::PreparedLinks,
    pub files: Vec<PathBuf>,
    pub graph_files: Vec<PathBuf>,
    pub graph_files_complete: bool,
    pub queue_paths: Option<super::family_scope::FamilyPathScope>,
    pub ts: FileIdMap<Arc<CheckFileFacts>>,
    pub(crate) vitest_exclude_imports:
        crate::codebase::testing_analysis::vitest_excludes::ResolvedSpreads,
    pub call_site_paths: Arc<std::collections::BTreeSet<PathBuf>>,
    pub graph_plan: crate::codebase::ts_source::facts::TsFactPlan,
    pub(crate) integration_runner_configs: std::collections::BTreeMap<
        PathBuf,
        crate::integration_tests::runner_config::RunnerConfigFileFacts,
    >,
    pub playwright_source_files: Arc<Vec<PathBuf>>,
    pub(crate) playwright_test_files_by_project: PlaywrightTestFilesByProject,
    pub playwright_matcher_warnings: Arc<std::collections::BTreeMap<Option<String>, Vec<String>>>,
    pub stats: CheckFactStats,
    /// Deduped SQL-looking member `.query` calls that were not proven executors.
    /// Printed by the CLI; it is not a finding and does not fail the run.
    pub unproven_member_query_sql: u32,
    pub app_selector_occurrences_cache: AppSelectorOccurrencesCache,
    pub playwright_routes_cache: PlaywrightRoutesCache,
    pub(crate) app_text_targets_cache: AppTextTargetsCache,
    pub route_reachable_files_cache: RouteReachableFilesCache,
}

impl CheckFactMap {
    pub fn register_fact_panics<'a>(
        &self,
        session: &crate::codebase::analysis_session::AnalysisSession,
        paths: impl IntoIterator<Item = &'a PathBuf>,
    ) {
        for path in paths {
            if let Some(error) = self
                .ts
                .get(path)
                .filter(|facts| facts.fact_extraction_panicked)
                .and_then(|facts| facts.ts.operational_error.as_deref())
            {
                crate::codebase::fact_panic::record(session, path, error);
            }
        }
    }

    /// SQL-looking member `.query` calls that were not proven to be executors.
    #[doc(hidden)]
    pub fn unproven_member_query_sql(&self) -> u32 {
        self.unproven_member_query_sql
    }

    pub fn queue_source_files(&self) -> Option<Vec<PathBuf>> {
        let scope = self.queue_paths.as_ref()?;
        Some(
            self.files
                .iter()
                .filter(|path| {
                    matches!(
                        path.extension().and_then(|ext| ext.to_str()),
                        Some("ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs")
                    ) && super::family_scope::applies(scope, path)
                })
                .cloned()
                .collect(),
        )
    }
}

#[derive(Default)]
pub struct CheckFileFacts {
    pub fact_extraction_panicked: bool,
    pub(crate) nested_controls: Option<Vec<crate::codebase::rules::nested_controls::Location>>,
    pub(crate) test_reliability: Vec<crate::codebase::testing_analysis::reliability::Location>,
    pub(crate) fixture_initializers: Option<
        crate::codebase::testing_analysis::reliability::playwright_provenance::FixtureInitializers,
    >,
    pub(crate) vitest_default_excludes:
        crate::codebase::testing_analysis::vitest_excludes::CollectedExcludes,
    pub(crate) csp_allowed_origins: crate::codebase::rules::csp_allowed_origins::CollectedOrigins,
    pub(crate) sql_source_prerequisites: super::SqlSourcePrerequisites,
    pub ts: Arc<TsFileFacts>,
    pub(crate) lint_suppressions:
        Option<Vec<crate::codebase::rules::forbidden_lint_suppressions::Suppression>>,
    pub production_imports:
        Option<Arc<Vec<crate::codebase::dependencies::extract::ExtractedImport>>>,
    pub source: Option<Arc<str>>,
    pub symbols: Option<Arc<FileSymbols>>,
    pub legacy_symbols: Option<Arc<FileSymbols>>,
    pub(crate) react: Option<Arc<ReactFileAnalysis>>,
    pub react_usages: Option<UsageFileFacts>,
    pub(crate) integration: Option<IntegrationFileAnalysis>,
    pub(crate) integration_runner_config:
        Option<crate::integration_tests::runner_config::RunnerConfigFileFacts>,
    pub dynamic_imports: Option<TestFacts>,
    pub(crate) nextjs_caching: Option<Vec<NextjsCachingFinding>>,
    pub(crate) nextjs_caching_low: Option<Vec<NextjsCachingFinding>>,
    pub(crate) storybook: Option<StorybookFileFacts>,
    pub postgres_file: crate::codebase::postgres_analysis::FilePayload,
    pub(crate) server_route_client_boundary:
        Option<crate::codebase::rules::server_route_client_boundary::FileFacts>,
    pub(crate) playwright: Option<PlaywrightTestFacts>,
    pub(crate) playwright_fetch: Option<crate::fetch::file_facts::ParsedFileFacts>,
    pub playwright_app_selectors: HashMap<(PlaywrightSettingsKey, bool), Vec<AppSelector>>,
    pub(crate) playwright_app_text_targets: HashMap<PlaywrightSettingsKey, Vec<AppTextTarget>>,
    pub(crate) playwright_static_exports: Option<StaticExportValues>,
    pub(crate) integration_route_occurrences:
        Vec<crate::codebase::playwright_facts::integration::RouteOccurrence>,
    pub parse_error: Option<String>,
    pub legacy_symbol_parse_error: Option<String>,
    pub parsed: bool,
}
