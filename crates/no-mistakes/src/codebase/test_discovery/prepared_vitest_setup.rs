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
                    .filter_map(|setup| {
                        let path = setup.resolved_path.clone()?;
                        let field = match setup.field {
                            crate::integration_tests::types::VitestSetupField::SetupFiles => {
                                crate::codebase::dependencies::graph::VitestSetupField::SetupFiles
                            }
                            crate::integration_tests::types::VitestSetupField::GlobalSetup => {
                                crate::codebase::dependencies::graph::VitestSetupField::GlobalSetup
                            }
                        };
                        Some((path, field))
                    })
                    .collect::<Vec<_>>();
                setups.sort();
                setups.dedup();
                (!setups.is_empty()).then(|| {
                    crate::codebase::dependencies::graph::VitestSetupProject {
                        config: project.config.clone(),
                        scope: project.scope.clone(),
                        filter,
                        setups,
                    }
                })
            })
            .collect()
    }
}
