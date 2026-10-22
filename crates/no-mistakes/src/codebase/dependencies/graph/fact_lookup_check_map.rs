impl TsFactLookup for crate::codebase::check_facts::CheckFactMap {
    fn companion_import_plans(
        &self,
    ) -> &[Arc<crate::codebase::rules::required_companion_imports::PreparedCompanionImportPlan>]
    {
        &self.companion_import_plans
    }
    fn get_production_imports(&self, path: &Path) -> Option<&[ExtractedImport]> {
        fact_lookup_production::from_check(self, path)
    }
    fn integration_route_links(
        &self,
        source: &crate::config::v2::schema::RouteCoverageSource,
    ) -> Option<&Result<Vec<crate::playwright::integration_routes::RouteLink>, String>> {
        self.integration_route_links.get(source)
    }
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        check_file_facts_for_path(self, path).map(|facts| facts.ts.as_ref())
    }

    fn covers_ts_fact_plan(&self, required: TsFactPlan) -> bool {
        self.graph_plan().covers(required)
    }

    fn covers_ts_fact_plan_for_file(&self, path: &Path, mut required: TsFactPlan) -> bool {
        if required.call_sites
            && self
                .call_site_paths
                .binary_search_by(|candidate| candidate.as_path().cmp(path))
                .is_ok()
        {
            required.call_sites = false;
        }
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
