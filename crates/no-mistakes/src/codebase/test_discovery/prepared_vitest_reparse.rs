impl PreparedTestProjects {
    /// A workspace string can lead to a standalone config below a package
    /// tsconfig. Reparse Vitest once after those parsed scopes seed the final
    /// catalog, reusing the request source store instead of rediscovering files.
    pub(crate) fn reparse_vitest_with_final_catalog(
        &mut self,
        root: &std::path::Path,
        config: &crate::config::v2::schema::NoMistakesConfig,
        visible_paths: &[std::path::PathBuf],
        catalog: std::sync::Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
        sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
    ) {
        let Some(Ok(projects)) = self.projects.get(&TestRunner::Vitest) else {
            return;
        };
        // Only an unresolved bare static helper can become resolvable after
        // package project roots seed the final catalog. Dynamic expressions
        // and cyclic helpers retain their conservative initial facts.
        if !projects
            .iter()
            .flat_map(|project| &project.vitest_setup)
            .any(|setup| setup.needs_final_catalog_reparse)
        {
            return;
        }
        let runner_configs = crate::integration_tests::runner_config::prepare_with_catalog_and_sources(
            root,
            config,
            visible_paths,
            std::sync::Arc::clone(&catalog),
            std::sync::Arc::clone(&sources),
        );
        let (projects, _) = runner_configs.with_request_cache_and_sources(None, Some(sources), || {
            projects::runner_projects_from_visible_with_catalog(
                root,
                config,
                TestRunner::Vitest,
                visible_paths,
                &catalog,
            )
            .map_err(|error| format!("{error:#}"))
        });
        self.projects.insert(TestRunner::Vitest, projects);
    }
}
