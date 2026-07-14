/// Runner projects parsed once for a request and reused by graph test filters
/// and every framework-specific discovery view.
#[doc(hidden)]
pub struct PreparedTestProjects {
    projects: BTreeMap<TestRunner, std::result::Result<Vec<ConfigProject>, String>>,
    graph_facts: crate::codebase::ts_source::facts::TsFactMap,
}

#[doc(hidden)]
pub(crate) fn prepare_test_projects_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    graph_indexable_files: &[PathBuf],
    graph_plan: crate::codebase::ts_source::facts::TsFactPlan,
    graph_context: crate::codebase::ts_source::facts::TsFactContext,
) -> PreparedTestProjects {
    let runner_configs =
        crate::integration_tests::runner_config::prepare(root, config, visible_paths, tsconfig);
    let runner_fact_plan = crate::integration_tests::runner_config::RunnerConfigFactPlan {
        root: root.to_path_buf(),
        primary_files: std::collections::HashSet::new(),
        graph_files: graph_indexable_files.iter().cloned().collect(),
        primary_plan: crate::codebase::check_facts::CheckFactPlan::default(),
        graph_plan: crate::codebase::check_facts::CheckFactPlan {
            graph: graph_plan,
            graph_context,
            ..Default::default()
        },
        playwright: None,
    };
    let (projects, helper_facts) = runner_configs.with_request_cache(Some(runner_fact_plan), || {
        [TestRunner::Dotnet, TestRunner::Vitest, TestRunner::Playwright, TestRunner::Swift]
            .into_iter()
            .map(|runner| {
                let projects = projects::runner_projects_from_visible(
                    root, config, runner, visible_paths, tsconfig,
                )
                .map_err(|error| format!("{error:#}"));
                (runner, projects)
            })
            .collect()
    });
    PreparedTestProjects {
        projects,
        graph_facts: crate::codebase::ts_source::facts::TsFactMap::from_iter_with_plan(
            helper_facts.into_iter().map(|(path, facts)| (path, facts.ts)),
            graph_plan,
        ),
    }
}

impl PreparedTestProjects {
    fn projects(&self, runner: TestRunner) -> Result<Vec<ConfigProject>> {
        self.projects
            .get(&runner)
            .expect("every runner is prepared")
            .clone()
            .map_err(anyhow::Error::msg)
    }

    #[doc(hidden)]
    pub fn project_filters(&self) -> Vec<(TestRunner, ProjectTestFilter)> {
        self.projects
            .iter()
            .filter_map(|(runner, projects)| projects.as_ref().ok().map(|projects| (*runner, projects)))
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
}
