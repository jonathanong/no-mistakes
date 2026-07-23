impl PreparedTestProjects {
    fn requested_projects(&self, runner: TestRunner) -> Option<Result<Vec<ConfigProject>>> {
        self.projects
            .get(&runner)
            .cloned()
            .map(|projects| projects.map_err(anyhow::Error::msg))
    }

    fn projects_if_prepared(&self, runner: TestRunner) -> Option<Vec<ConfigProject>> {
        self.projects
            .get(&runner)
            .and_then(|projects| projects.as_ref().ok())
            .cloned()
    }

    /// Return the runner projects already parsed for this request. This is a
    /// catalog view only: callers must not cause a second runner-config parse.
    #[doc(hidden)]
    pub fn requested_runner_projects(
        &self,
        runner: TestRunner,
    ) -> Result<Vec<crate::codebase::test_discovery::PreparedRunnerProject>> {
        self.requested_projects(runner)
            .transpose()?
            .ok_or_else(|| anyhow::anyhow!("{} runner projects were not prepared", runner.as_str()))
            .map(|projects| {
                projects
                    .into_iter()
                    .map(|project| crate::codebase::test_discovery::PreparedRunnerProject {
                        config: project.config,
                        runner_project_arg: project.runner_project_arg,
                    })
                    .collect()
            })
    }

    #[doc(hidden)]
    pub fn project_filters(&self) -> Vec<(TestRunner, ProjectTestFilter)> {
        self.projects
            .iter()
            .filter_map(|(runner, projects)| {
                projects.as_ref().ok().map(|projects| (*runner, projects))
            })
            .flat_map(|(runner, projects)| {
                projects
                    .iter()
                    .filter_map(|project| ProjectTestFilter::from_project_ref(project).ok())
                    .map(move |filter| (runner, filter))
            })
            .collect()
    }

    #[doc(hidden)]
    pub fn graph_facts(&self) -> &crate::codebase::ts_source::facts::TsFactMap {
        &self.graph_facts
    }

    /// Parsed runner projects retained for request-scoped graph features.
    /// Callers must not fall back to reparsing configuration when this returns
    /// `None`; the runner was simply not requested for this invocation.
    #[doc(hidden)]
    pub(crate) fn prepared_projects(&self, runner: TestRunner) -> Option<&[ConfigProject]> {
        self.projects
            .get(&runner)
            .and_then(|projects| projects.as_ref().ok())
            .map(Vec::as_slice)
    }

    pub(crate) fn dotnet_facts(&self) -> Option<&crate::codebase::dotnet::DotnetFactMap> {
        self.dotnet_facts.as_ref()
    }

    pub(crate) fn swift_facts(&self) -> Option<&crate::codebase::swift::SwiftFactMap> {
        self.swift_facts.as_ref()
    }

    #[doc(hidden)]
    pub fn tsconfig_candidate_roots(&self, root: &Path) -> Vec<PathBuf> {
        let mut roots = self
            .projects
            .values()
            .filter_map(|projects| projects.as_ref().ok())
            .flat_map(|projects| projects.iter())
            .flat_map(|project| {
                let scope = project.scope.as_deref().map(|scope| root.join(scope));
                let config_dir = project
                    .config
                    .as_deref()
                    .map(|config| root.join(config))
                    .and_then(|config| config.parent().map(Path::to_path_buf));
                scope.into_iter().chain(config_dir)
            })
            .collect::<Vec<_>>();
        roots.sort();
        roots.dedup();
        roots
    }
}
