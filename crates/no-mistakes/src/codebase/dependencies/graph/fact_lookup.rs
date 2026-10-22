/// Shared, read-only lookup into already-collected TS facts, plus opt-in
/// memoization for a handful of app-wide Playwright scans that independent
/// call paths within one invocation would otherwise repeat (e.g. the
/// `playwright` rule and `forbidden-dependencies`'s `DepGraph` build each
/// analyze the whole app). See `crates/AGENTS.md`'s "Duplicate full-repo work
/// across independent call paths" section for the pattern this backs.
pub(crate) trait TsFactLookup: Sync {
    fn companion_import_plans(
        &self,
    ) -> &[Arc<crate::codebase::rules::required_companion_imports::PreparedCompanionImportPlan>]
    {
        &[]
    }
    fn integration_route_links(
        &self,
        _source: &crate::config::v2::schema::RouteCoverageSource,
    ) -> Option<&Result<Vec<crate::playwright::integration_routes::RouteLink>, String>> {
        None
    }
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts>;

    /// Production declaration checks use a distinct historical parser projection
    /// for files whose standard source type differs from TypeScript/TSX.
    fn get_production_imports(&self, path: &Path) -> Option<&[ExtractedImport]> {
        fact_lookup_production::from_ts(self.get_ts_facts(path)?, path)
    }

    /// Whether every returned TS fact was collected with at least this plan.
    /// A false result makes graph construction fill from source instead of
    /// mistaking present-but-empty sparse facts for complete import facts.
    fn covers_ts_fact_plan(&self, _required: TsFactPlan) -> bool {
        false
    }

    /// Per-file coverage for projections whose configured paths are narrower
    /// than the graph's all-file fact guarantee.
    fn covers_ts_fact_plan_for_file(&self, _path: &Path, required: TsFactPlan) -> bool {
        self.covers_ts_fact_plan(required)
    }

    /// Complete pre-discovered file universe associated with these facts.
    fn graph_files(&self) -> Option<&[PathBuf]> {
        None
    }

    /// Already-collected Playwright test-file facts (URLs, selectors, text
    /// locators, helper references), when available. Lets a consumer skip
    /// re-parsing/re-analyzing a test file it already has facts for.
    /// `TsFactMap` never carries these (only `CheckFactMap` does), so the
    /// default returns `None` — callers must already tolerate a per-file
    /// `None` by falling back to full analysis for that file.
    fn get_playwright_facts(
        &self,
        _path: &Path,
    ) -> Option<&crate::codebase::check_facts::PlaywrightTestFacts> {
        None
    }

    /// A cached parser diagnostic for a Playwright test file. Implementations
    /// that do not retain test parse failures return `None` and callers fall
    /// back to parsing the file normally.
    fn get_playwright_parse_error(&self, _path: &Path) -> Option<&str> {
        None
    }

    fn playwright_source_files(&self) -> Option<&[PathBuf]> {
        None
    }

    fn get_playwright_test_files(
        &self,
        _project: Option<&str>,
    ) -> Option<Arc<Vec<crate::playwright::analysis::context::DiscoveredTestFile>>> {
        None
    }

    fn get_playwright_fetch_facts(
        &self,
        _path: &Path,
    ) -> Option<Result<crate::fetch::file_facts::ParsedFileFacts, String>> {
        None
    }

    /// Get-or-compute the app-wide Playwright selector-occurrence scan
    /// (`collect_app_selector_occurrences`), keyed by the exact Playwright
    /// settings and `scan_html_ids`. A single request may analyze multiple
    /// projects with different roots, selector attributes, or rewrites; those
    /// scopes must never share cached app facts.
    ///
    /// Default: always calls `compute`, no caching — for implementors that do
    /// not share this scan across call sites. `TsFactMap` and `CheckFactMap`
    /// override this with an invocation-scoped DashMap.
    fn get_or_compute_app_selector_occurrences(
        &self,
        _settings: &crate::playwright::config::Settings,
        _scan_html_ids: bool,
        compute: &dyn Fn() -> Result<Vec<crate::playwright::selectors::AppSelector>>,
    ) -> Result<Arc<Vec<crate::playwright::selectors::AppSelector>>> {
        compute().map(Arc::new)
    }

    /// Get-or-compute the app's Playwright page routes (`routes::collect_routes`
    /// plus rewrite expansion), keyed by the exact Playwright settings so
    /// different frontend roots and rewrite sets remain isolated.
    fn get_or_compute_playwright_routes(
        &self,
        _settings: &crate::playwright::config::Settings,
        compute: &dyn Fn() -> Vec<crate::routes::Route>,
    ) -> Arc<Vec<crate::routes::Route>> {
        Arc::new(compute())
    }

    /// Get-or-compute the app-wide visible-text scan
    /// (`collect_app_text_targets`), keyed by exact Playwright settings.
    fn get_or_compute_app_text_targets(
        &self,
        _settings: &crate::playwright::config::Settings,
        compute: &dyn Fn() -> Result<Vec<crate::playwright::analysis::text_types::AppTextTarget>>,
    ) -> Result<Arc<Vec<crate::playwright::analysis::text_types::AppTextTarget>>> {
        compute().map(Arc::new)
    }

    /// Get-or-compute route-reachability (`collect_route_reachable_files`) —
    /// the single largest cost this cache eliminates in practice (roughly 8s
    /// per call on a large monorepo, paid twice without this). Depends on
    /// `root`, `settings`, and the already-shared routes list, so the cache is
    /// scoped by exact Playwright settings as well.
    fn get_or_compute_route_reachable_files(
        &self,
        _settings: &crate::playwright::config::Settings,
        compute: &dyn Fn() -> Result<RouteReachableFiles>,
    ) -> Result<Arc<RouteReachableFiles>> {
        compute().map(Arc::new)
    }
}

/// `app_file` → set of test-reachable source files that can navigate to it.
/// Named here (rather than inlined) because both the trait above and
/// `CheckFactMap`'s cache field need to name the exact same type.
pub(crate) type RouteReachableFiles =
    std::collections::BTreeMap<Arc<String>, std::collections::BTreeSet<Arc<String>>>;

mod fact_lookup_fallback;
mod fact_lookup_production;
pub(crate) use fact_lookup_fallback::*;
use fact_lookup_production::check_file_facts_for_path;

include!("fact_lookup_ts_map.rs");

include!("fact_lookup_check_map.rs");
