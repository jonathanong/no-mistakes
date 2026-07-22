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
            let project_root = project
                .scope
                .as_deref()
                .map(|scope| root.join(scope))
                .unwrap_or_else(|| root.to_path_buf());
            crate::integration_tests::resolve_setup_dependencies(
                project.vitest_setup.iter_mut(),
                &project_root,
                &resolver,
            );
        }
    }
}
