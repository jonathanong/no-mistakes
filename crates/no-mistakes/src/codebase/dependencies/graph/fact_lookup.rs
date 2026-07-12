/// Shared, read-only lookup into already-collected TS facts, plus opt-in
/// memoization for a handful of app-wide Playwright scans that independent
/// call paths within one invocation would otherwise repeat (e.g. the
/// `playwright` rule and `forbidden-dependencies`'s `DepGraph` build each
/// analyze the whole app). See `crates/CLAUDE.md`'s "Duplicate full-repo work
/// across independent call paths" section for the pattern this backs.
pub(crate) trait TsFactLookup: Sync {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts>;

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

    /// Get-or-compute the app-wide Playwright selector-occurrence scan
    /// (`collect_app_selector_occurrences`), keyed by `scan_html_ids` — whether
    /// HTML id attributes are included in the scan. Every other input to that
    /// scan (selector attributes, frontend root, selector roots/include/
    /// exclude) is resolved from one config file per `check` invocation and
    /// is therefore invariant across every caller within a run;
    /// `scan_html_ids` is the only value that can legitimately differ (e.g. a
    /// `playwright-unique-html-ids` rule selection scans with HTML ids
    /// included, while a `DepGraph` build for `forbidden-dependencies` never
    /// does), so a 2-entry cache fully captures the variance — a repo with
    /// that rule configured still pays the scan twice (once per key), by
    /// design, not by bug.
    ///
    /// Default: always calls `compute`, no caching — correct for `TsFactMap`
    /// and for any standalone single-invocation caller that has no reason to
    /// share this scan with another call site.
    fn get_or_compute_app_selector_occurrences(
        &self,
        _scan_html_ids: bool,
        compute: &dyn Fn() -> Result<Vec<crate::playwright::selectors::AppSelector>>,
    ) -> Result<Arc<Vec<crate::playwright::selectors::AppSelector>>> {
        compute().map(Arc::new)
    }

    /// Get-or-compute the app's Playwright page routes (`routes::collect_routes`
    /// plus rewrite expansion). Depends only on `root` and `settings.frontend_root`
    /// /`settings.rewrites`, neither of which varies by Playwright project —
    /// unlike `get_or_compute_app_selector_occurrences`, no key is needed:
    /// every caller within one invocation wants the same value.
    fn get_or_compute_playwright_routes(
        &self,
        compute: &dyn Fn() -> Vec<crate::routes::Route>,
    ) -> Arc<Vec<crate::routes::Route>> {
        Arc::new(compute())
    }

    /// Get-or-compute the app-wide visible-text scan (`collect_app_text_targets`).
    /// Same invariance argument as `get_or_compute_playwright_routes`.
    fn get_or_compute_app_text_targets(
        &self,
        compute: &dyn Fn() -> Result<Vec<crate::playwright::analysis::text_types::AppTextTarget>>,
    ) -> Result<Arc<Vec<crate::playwright::analysis::text_types::AppTextTarget>>> {
        compute().map(Arc::new)
    }

    /// Get-or-compute route-reachability (`collect_route_reachable_files`) —
    /// the single largest cost this cache eliminates in practice (roughly 8s
    /// per call on a large monorepo, paid twice without this). Depends on
    /// `root`, `settings`, and the already-shared routes list; same
    /// invariance argument as the other keyless caches above.
    fn get_or_compute_route_reachable_files(
        &self,
        compute: &dyn Fn() -> Result<RouteReachableFiles>,
    ) -> Result<Arc<RouteReachableFiles>> {
        compute().map(Arc::new)
    }
}

/// `app_file` → set of test-reachable source files that can navigate to it.
/// Named here (rather than inlined) because both the trait above and
/// `CheckFactMap`'s cache field need to name the exact same type.
pub(crate) type RouteReachableFiles = std::collections::BTreeMap<Arc<String>, std::collections::BTreeSet<Arc<String>>>;

impl TsFactLookup for TsFactMap {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        self.get(path)
    }
}

impl TsFactLookup for crate::codebase::check_facts::CheckFactMap {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        self.ts.get(path).map(|facts| &facts.ts)
    }

    fn get_playwright_facts(
        &self,
        path: &Path,
    ) -> Option<&crate::codebase::check_facts::PlaywrightTestFacts> {
        self.ts
            .get(path)
            .and_then(|facts| facts.playwright.as_ref())
    }

    fn get_or_compute_app_selector_occurrences(
        &self,
        scan_html_ids: bool,
        compute: &dyn Fn() -> Result<Vec<crate::playwright::selectors::AppSelector>>,
    ) -> Result<Arc<Vec<crate::playwright::selectors::AppSelector>>> {
        if let Some(cached) = self.app_selector_occurrences_cache.get(&scan_html_ids) {
            return Ok(cached.clone());
        }
        // As with the OnceLock-backed caches below, the fallible compute
        // happens outside any lock: `compute` may run more than once under a
        // genuine race, but `entry(..).or_insert_with(..)` still inserts
        // exactly once and every caller ends up with the same cached Arc.
        let computed = Arc::new(compute()?);
        Ok(self
            .app_selector_occurrences_cache
            .entry(scan_html_ids)
            .or_insert_with(|| computed.clone())
            .clone())
    }

    fn get_or_compute_playwright_routes(
        &self,
        compute: &dyn Fn() -> Vec<crate::routes::Route>,
    ) -> Arc<Vec<crate::routes::Route>> {
        self.playwright_routes_cache
            .get_or_init(|| Arc::new(compute()))
            .clone()
    }

    fn get_or_compute_app_text_targets(
        &self,
        compute: &dyn Fn() -> Result<Vec<crate::playwright::analysis::text_types::AppTextTarget>>,
    ) -> Result<Arc<Vec<crate::playwright::analysis::text_types::AppTextTarget>>> {
        // `OnceLock::get_or_try_init` is not yet stable, so the fallible
        // compute happens outside the lock: `compute` may run more than once
        // under a genuine race, but it's deterministic, so a benign race just
        // means both threads agree on the value that ends up cached.
        if let Some(cached) = self.app_text_targets_cache.get() {
            return Ok(cached.clone());
        }
        let computed = Arc::new(compute()?);
        Ok(self
            .app_text_targets_cache
            .get_or_init(|| computed.clone())
            .clone())
    }

    fn get_or_compute_route_reachable_files(
        &self,
        compute: &dyn Fn() -> Result<RouteReachableFiles>,
    ) -> Result<Arc<RouteReachableFiles>> {
        if let Some(cached) = self.route_reachable_files_cache.get() {
            return Ok(cached.clone());
        }
        let computed = Arc::new(compute()?);
        Ok(self
            .route_reachable_files_cache
            .get_or_init(|| computed.clone())
            .clone())
    }
}
