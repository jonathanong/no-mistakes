use super::CheckFactMap;
use std::path::PathBuf;
use std::sync::Arc;

impl CheckFactMap {
    pub(super) fn view_with_supplemental(
        &self,
        supplemental: &Self,
        graph_files: Vec<PathBuf>,
    ) -> Self {
        let mut ts = self.ts.clone();
        for (path, facts) in supplemental.ts.iter() {
            ts.insert(path.clone(), Arc::clone(facts));
        }
        let mut graph_plan = self.graph_plan;
        graph_plan.include(supplemental.graph_plan);
        let unproven_member_query_sql =
            crate::codebase::check_facts::relative_scope::count_unproven_member_queries(
                ts.values().map(|facts| facts.as_ref()),
            );
        Self {
            companion_import_plans: crate::codebase::rules::required_companion_imports::merge_plans(
                &self.companion_import_plans,
                &supplemental.companion_import_plans,
            ),
            postgres: self.postgres.clone(),
            files: self.files.clone(),
            integration_route_links: self.integration_route_links.clone(),
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
            postgres_schema_catalogs: self.postgres_schema_catalogs.clone(),
            stats: self.stats,
            unproven_member_query_sql,
        }
    }
}
