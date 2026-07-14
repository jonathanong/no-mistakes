/// Selects which framework project configurations are prepared for one request.
///
/// Keeping this separate from the graph fact plan makes it possible for an
/// import-only request to avoid touching unrelated test runner configuration.
#[doc(hidden)]
#[derive(Debug, Clone, Default)]
pub struct FrameworkPreparationPlan {
    runners: std::collections::BTreeSet<TestRunner>,
}

impl FrameworkPreparationPlan {
    #[doc(hidden)]
    pub fn all() -> Self {
        Self {
            runners: [
                TestRunner::Dotnet,
                TestRunner::Playwright,
                TestRunner::Vitest,
                TestRunner::Swift,
            ]
            .into_iter()
            .collect(),
        }
    }

    #[doc(hidden)]
    pub fn for_runners(runners: impl IntoIterator<Item = TestRunner>) -> Self {
        Self {
            runners: runners.into_iter().collect(),
        }
    }

    pub(crate) fn for_graph(plan: crate::codebase::dependencies::graph::GraphBuildPlan) -> Self {
        let mut demand = Self::default();
        // TestOf classification is defined by the union of every configured
        // suite, so this relationship intentionally prepares every runner.
        if plan.tests {
            demand = Self::all();
        }
        demand
    }

    pub(crate) fn include_framework_names<'a>(&mut self, names: impl IntoIterator<Item = &'a str>) {
        for name in names {
            if let Some(runner) = TestRunner::from_name(name) {
                self.insert(runner);
            }
        }
    }

    pub(crate) fn include(&mut self, other: Self) {
        self.runners.extend(other.runners);
    }

    fn insert(&mut self, runner: TestRunner) {
        self.runners.insert(runner);
    }

    fn runners(&self) -> impl Iterator<Item = TestRunner> + '_ {
        self.runners.iter().copied()
    }

    pub(crate) fn excluded_config_paths(
        &self,
        root: &Path,
        config: &NoMistakesConfig,
        visible_paths: &[PathBuf],
    ) -> std::collections::HashSet<PathBuf> {
        if self.runners.is_empty() {
            return std::collections::HashSet::new();
        }
        [TestRunner::Playwright, TestRunner::Vitest]
            .into_iter()
            .filter(|runner| !self.runners.contains(runner))
            .flat_map(|runner| {
                let (configured, _) = projects::runner_config(config, runner);
                configured.map_or_else(
                    || {
                        crate::integration_tests::project_config::discovered_config_paths(
                            root,
                            runner.framework(),
                            visible_paths,
                        )
                    },
                    crate::config::v2::schema::StringOrList::values,
                )
            })
            .map(|path| crate::codebase::ts_resolver::normalize_path(&root.join(path)))
            .collect()
    }

    #[doc(hidden)]
    pub fn contains(&self, runner: TestRunner) -> bool {
        self.runners.contains(&runner)
    }
}
