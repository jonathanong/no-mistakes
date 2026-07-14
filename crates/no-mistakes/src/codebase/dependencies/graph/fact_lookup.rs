/// Shared, read-only lookup into already-collected TS facts, plus opt-in
/// memoization for a handful of app-wide Playwright scans that independent
/// call paths within one invocation would otherwise repeat (e.g. the
/// `playwright` rule and `forbidden-dependencies`'s `DepGraph` build each
/// analyze the whole app). See `crates/CLAUDE.md`'s "Duplicate full-repo work
/// across independent call paths" section for the pattern this backs.
pub(crate) trait TsFactLookup: Sync {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts>;

    /// Whether every returned TS fact was collected with at least this plan.
    /// A false result makes graph construction fill from source instead of
    /// mistaking present-but-empty sparse facts for complete import facts.
    fn covers_ts_fact_plan(&self, _required: TsFactPlan) -> bool {
        false
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
    /// Default: always calls `compute`, no caching — correct for `TsFactMap`
    /// and for any standalone single-invocation caller that has no reason to
    /// share this scan with another call site.
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

include!("fact_lookup_fallback.rs");

/// `app_file` → set of test-reachable source files that can navigate to it.
/// Named here (rather than inlined) because both the trait above and
/// `CheckFactMap`'s cache field need to name the exact same type.
pub(crate) type RouteReachableFiles =
    std::collections::BTreeMap<Arc<String>, std::collections::BTreeSet<Arc<String>>>;

impl TsFactLookup for TsFactMap {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        self.get(path)
    }

    fn covers_ts_fact_plan(&self, required: TsFactPlan) -> bool {
        self.plan().covers(required)
    }
}

fn check_file_facts_for_path<'a>(
    facts: &'a crate::codebase::check_facts::CheckFactMap,
    path: &Path,
) -> Option<&'a std::sync::Arc<crate::codebase::check_facts::CheckFileFacts>> {
    facts.ts.get(path).or_else(|| {
        let normalized = crate::codebase::ts_resolver::normalize_path(path);
        facts.ts.get(&normalized)
    })
}

impl TsFactLookup for crate::codebase::check_facts::CheckFactMap {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        check_file_facts_for_path(self, path).map(|facts| facts.ts.as_ref())
    }

    fn covers_ts_fact_plan(&self, required: TsFactPlan) -> bool {
        self.graph_plan().covers(required)
    }

    fn graph_files(&self) -> Option<&[PathBuf]> {
        self.graph_files_complete
            .then_some(self.graph_files.as_slice())
    }

    fn get_playwright_facts(
        &self,
        path: &Path,
    ) -> Option<&crate::codebase::check_facts::PlaywrightTestFacts> {
        check_file_facts_for_path(self, path).and_then(|facts| facts.playwright.as_ref())
    }

    fn playwright_source_files(&self) -> Option<&[PathBuf]> {
        Some(&self.playwright_source_files)
    }

    fn get_playwright_test_files(
        &self,
        project: Option<&str>,
    ) -> Option<Arc<Vec<crate::playwright::analysis::context::DiscoveredTestFile>>> {
        self.playwright_test_files_by_project
            .iter()
            .find(|(candidate, _)| candidate.as_deref() == project)
            .map(|(_, files)| Arc::clone(files))
    }

    fn get_playwright_fetch_facts(
        &self,
        path: &Path,
    ) -> Option<Result<crate::fetch::file_facts::ParsedFileFacts, String>> {
        let facts = check_file_facts_for_path(self, path)?;
        if let Some(error) = &facts.parse_error {
            return Some(Err(format!("failed to parse {}: {error}", path.display())));
        }
        facts.playwright_fetch.as_ref().cloned().map(Ok)
    }

    fn get_playwright_parse_error(&self, path: &Path) -> Option<&str> {
        check_file_facts_for_path(self, path).and_then(|facts| facts.parse_error.as_deref())
    }

    fn get_or_compute_app_selector_occurrences(
        &self,
        settings: &crate::playwright::config::Settings,
        scan_html_ids: bool,
        compute: &dyn Fn() -> Result<Vec<crate::playwright::selectors::AppSelector>>,
    ) -> Result<Arc<Vec<crate::playwright::selectors::AppSelector>>> {
        // `OnceLock::get_or_try_init` is not yet stable, so a fallible
        // `entry(..).or_insert_with(..)` closure is the only way to keep the
        // *compute itself* — not just the insert — inside the per-key lock:
        // errors are cached as `String` (`anyhow::Error` isn't `Clone`) so two
        // concurrent misses can't both pay the full scan before either caches
        // it.
        self.app_selector_occurrences_cache
            .entry((
                crate::codebase::check_facts::PlaywrightSettingsKey::new(settings),
                scan_html_ids,
            ))
            .or_insert_with(|| {
                compute()
                    .map(Arc::new)
                    .map_err(|error| format!("{error:#}"))
            })
            .clone()
            .map_err(anyhow::Error::msg)
    }

    fn get_or_compute_playwright_routes(
        &self,
        settings: &crate::playwright::config::Settings,
        compute: &dyn Fn() -> Vec<crate::routes::Route>,
    ) -> Arc<Vec<crate::routes::Route>> {
        self.playwright_routes_cache
            .entry(crate::codebase::check_facts::PlaywrightSettingsKey::new(
                settings,
            ))
            .or_insert_with(|| Arc::new(compute()))
            .clone()
    }

    fn get_or_compute_app_text_targets(
        &self,
        settings: &crate::playwright::config::Settings,
        compute: &dyn Fn() -> Result<Vec<crate::playwright::analysis::text_types::AppTextTarget>>,
    ) -> Result<Arc<Vec<crate::playwright::analysis::text_types::AppTextTarget>>> {
        self.app_text_targets_cache
            .entry(crate::codebase::check_facts::PlaywrightSettingsKey::new(
                settings,
            ))
            .or_insert_with(|| {
                compute()
                    .map(Arc::new)
                    .map_err(|error| format!("{error:#}"))
            })
            .clone()
            .map_err(anyhow::Error::msg)
    }

    fn get_or_compute_route_reachable_files(
        &self,
        settings: &crate::playwright::config::Settings,
        compute: &dyn Fn() -> Result<RouteReachableFiles>,
    ) -> Result<Arc<RouteReachableFiles>> {
        self.route_reachable_files_cache
            .entry(crate::codebase::check_facts::PlaywrightSettingsKey::new(
                settings,
            ))
            .or_insert_with(|| {
                compute()
                    .map(Arc::new)
                    .map_err(|error| format!("{error:#}"))
            })
            .clone()
            .map_err(anyhow::Error::msg)
    }
}
