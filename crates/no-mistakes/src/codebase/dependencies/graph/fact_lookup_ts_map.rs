impl TsFactLookup for TsFactMap {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        self.get(path)
    }

    fn covers_ts_fact_plan(&self, required: TsFactPlan) -> bool {
        self.plan().covers(required)
    }

    fn get_or_compute_app_selector_occurrences(
        &self,
        settings: &crate::playwright::config::Settings,
        scan_html_ids: bool,
        compute: &dyn Fn() -> Result<Vec<crate::playwright::selectors::AppSelector>>,
    ) -> Result<Arc<Vec<crate::playwright::selectors::AppSelector>>> {
        self.app_selector_occurrences_cache
            .entry((self.playwright_scan_cache_key(settings), scan_html_ids))
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
            .entry(self.playwright_scan_cache_key(settings))
            .or_insert_with(|| Arc::new(compute()))
            .clone()
    }

    fn get_or_compute_app_text_targets(
        &self,
        settings: &crate::playwright::config::Settings,
        compute: &dyn Fn() -> Result<Vec<crate::playwright::analysis::text_types::AppTextTarget>>,
    ) -> Result<Arc<Vec<crate::playwright::analysis::text_types::AppTextTarget>>> {
        self.app_text_targets_cache
            .entry(self.playwright_scan_cache_key(settings))
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
        // Settings alone are not enough: SharedTraversalContext can extend
        // this map, add explicit roots, and rebuild the graph. Generation is
        // part of the key so a later universe cannot reuse the prior scan.
        self.route_reachable_files_cache
            .entry(self.playwright_scan_cache_key(settings))
            .or_insert_with(|| {
                compute()
                    .map(Arc::new)
                    .map_err(|error| format!("{error:#}"))
            })
            .clone()
            .map_err(anyhow::Error::msg)
    }
}
