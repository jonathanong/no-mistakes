impl PreparedTestProjects {
    /// Re-resolve setup modules after parsed project scopes seed the final
    /// importer-scoped catalog. Runner config source/AST facts remain cached;
    /// this only refreshes resolver-derived setup paths and candidates.
    pub(crate) fn reresolve_vitest_setups(
        &mut self,
        root: &std::path::Path,
        catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
        visible_paths: &[std::path::PathBuf],
    ) {
        let Some(Ok(projects)) = self.projects.get_mut(&TestRunner::Vitest) else {
            return;
        };
        let visible = visible_paths.iter().cloned().collect();
        let resolver = crate::codebase::ts_resolver::ScopedImportResolver::from_visible(
            catalog, &visible,
        );
        for project in projects {
            let fallback_root = project
                .scope
                .as_deref()
                .map(|scope| root.join(scope))
                .unwrap_or_else(|| root.to_path_buf());
            for setup in &mut project.vitest_setup {
                // Explicit policies intentionally clear the project scope.
                // Parsed setup declarations retain their original effective
                // root, which is the only sound base for a re-resolution.
                let resolution_base = if setup.resolution_base.as_os_str().is_empty() {
                    fallback_root.clone()
                } else {
                    setup.resolution_base.clone()
                };
                crate::integration_tests::resolve_setup_dependencies(
                    std::iter::once(setup),
                    &resolution_base,
                    &resolver,
                );
            }
        }
    }
}
