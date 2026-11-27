impl PreparedGraphConfig {
    #[doc(hidden)]
    pub fn playwright_fact_plan_partial(
        &mut self,
        root: &Path,
        tsconfig: &crate::codebase::ts_resolver::TsConfig,
        visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    ) -> (
        Option<crate::codebase::check_facts::PlaywrightFactPlan>,
        Vec<PlaywrightPreparationFailure>,
    ) {
        let prepare_playwright = crate::codebase::playwright_hooks::hooks().partial_graph_fact_plan;
        let (mut merged, failures) = prepare_playwright(
            root,
            &mut self.playwright_settings,
            visible_paths,
            std::mem::take(&mut self.playwright_preparation_failures),
        );
        if self.playwright_settings.is_empty() && !failures.is_empty() {
            self.disable_playwright();
        }
        if let Some(plan) = merged.as_mut() {
            plan.configure_module_resolution(
                std::sync::Arc::new(tsconfig.clone()),
                std::sync::Arc::clone(&self.workspace),
                visible_paths,
                root,
            );
        }
        (merged, failures)
    }
}
