impl PreparedTestProjects {
    /// Keep project ownership compact; the graph scans its existing file
    /// universe only when a traversal first requests test adjacency.
    pub(crate) fn vitest_setup_projects(
        &self,
    ) -> Vec<crate::codebase::dependencies::graph::VitestSetupProject> {
        let Some(projects) = self.prepared_projects(TestRunner::Vitest) else {
            return Vec::new();
        };
        projects
            .iter()
            .filter_map(|project| {
                let filter = ProjectTestFilter::from_project_ref(project).ok()?;
                let mut setups = project
                    .vitest_setup
                    .iter()
                    .filter(|setup| setup.specifier.is_some())
                    .flat_map(|setup| {
                        let field = match setup.field {
                            crate::integration_tests::types::VitestSetupField::SetupFiles => {
                                crate::codebase::dependencies::graph::VitestSetupField::SetupFiles
                            }
                            crate::integration_tests::types::VitestSetupField::GlobalSetup => {
                                crate::codebase::dependencies::graph::VitestSetupField::GlobalSetup
                            }
                        };
                        setup
                            .resolved_path
                            .iter()
                            .chain(
                                setup
                                    .resolved_path
                                    .is_none()
                                    .then_some(&setup.resolver_candidate_paths)
                                    .into_iter()
                                    .flatten(),
                            )
                            .cloned()
                            .map(move |path| (path, field))
                    })
                    .collect::<Vec<_>>();
                setups.sort();
                setups.dedup();
                (!setups.is_empty()).then(|| {
                    let tests: Vec<_> = crate::codebase::ts_source::discover_files_from_visible(
                        &self.root,
                        &self.skip_directories,
                        &self.visible_paths,
                    )
                    .into_iter()
                        .filter(|path| {
                            let relative = crate::codebase::ts_source::relative_slash_path(
                                &self.root,
                                path,
                            );
                            filter.is_match(&relative)
                        })
                        .collect();
                    (!tests.is_empty()).then(|| crate::codebase::dependencies::graph::VitestSetupProject {
                        config: project.config.clone(),
                        scope: project.scope.clone(),
                        filter,
                        tests,
                        setups,
                    })
                }).flatten()
            })
            .collect()
    }
}
