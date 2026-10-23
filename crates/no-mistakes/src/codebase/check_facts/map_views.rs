use super::CheckFactMap;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(not(test))]
macro_rules! record_view_composition {
    () => {};
}

#[cfg(test)]
#[macro_use]
#[path = "map/composition_metrics_tests.rs"]
mod composition_metrics_tests;

impl CheckFactMap {
    pub(super) fn view_with_supplemental(
        &self,
        supplemental: &Self,
        graph_files: Vec<PathBuf>,
    ) -> Self {
        self.compose_view(supplemental, graph_files, true)
    }

    /// Merge supplemental facts the way the CLI does: fill keys that are
    /// absent, and keep a richer primary entry such as production imports.
    pub(crate) fn fact_view_filling_missing(&self, supplemental: &Self) -> Self {
        self.compose_view(supplemental, self.graph_files.clone(), false)
    }

    fn compose_view(
        &self,
        supplemental: &Self,
        graph_files: Vec<PathBuf>,
        overwrite: bool,
    ) -> Self {
        record_view_composition!();
        let mut ts = self.ts.clone();
        for (path, facts) in supplemental.ts.iter() {
            if overwrite || !ts.contains_key(path) {
                ts.insert(path.clone(), Arc::clone(facts));
            }
        }
        let mut call_site_paths = Arc::clone(&self.call_site_paths);
        // Scoped coverage follows the winning entry, including overwrites that replace
        // call summaries with an imports-only projection. Disjoint overlays share.
        let loses_coverage = |path: &PathBuf| {
            supplemental.ts.contains_key(path) && !supplemental.call_site_paths.contains(path)
        };
        if overwrite
            && !supplemental.graph_plan.call_sites
            && call_site_paths.iter().any(loses_coverage)
        {
            Arc::make_mut(&mut call_site_paths).retain(|path| !loses_coverage(path));
        }
        // A rejected sparse duplicate cannot add coverage to the richer winner.
        let supplemental_wins = |path: &&PathBuf| overwrite || !self.ts.contains_key(path);
        if !Arc::ptr_eq(&call_site_paths, &supplemental.call_site_paths)
            && supplemental
                .call_site_paths
                .iter()
                .filter(supplemental_wins)
                .any(|path| !call_site_paths.contains(path))
        {
            if call_site_paths.is_empty()
                && supplemental
                    .call_site_paths
                    .iter()
                    .all(|path| overwrite || !self.ts.contains_key(path))
            {
                call_site_paths = Arc::clone(&supplemental.call_site_paths);
            } else {
                Arc::make_mut(&mut call_site_paths).extend(
                    supplemental
                        .call_site_paths
                        .iter()
                        .filter(supplemental_wins)
                        .cloned(),
                );
            }
        }
        let mut graph_plan = self.graph_plan;
        graph_plan.include(supplemental.graph_plan);
        let unproven_member_query_sql =
            crate::codebase::check_facts::relative_scope::count_unproven_member_queries(
                ts.values().map(|facts| facts.as_ref()),
            );
        Self {
            sql_application: self.sql_application.clone(),
            sql_verdicts: self.sql_verdicts.clone(),
            sql_advisories: Arc::clone(&self.sql_advisories),
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
            call_site_paths,
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
