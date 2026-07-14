pub fn discover_tests(root: &Path, config: &NoMistakesConfig, runner: TestRunner) -> Result<DiscoveredTests> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, root, &visible_paths)?;
    discover_tests_from_visible(root, config, runner, &visible_paths, &tsconfig)
}

#[doc(hidden)]
pub fn discover_tests_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> Result<DiscoveredTests> {
    let projects = projects::runner_projects_from_visible(root, config, runner, visible_paths, tsconfig)?;
    discover_from_projects_from_visible(
        root,
        config,
        runner,
        projects,
        None,
        visible_paths,
        tsconfig,
    )
}

#[doc(hidden)]
pub fn discover_tests_from_prepared_projects(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    prepared: &PreparedTestProjects,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> Result<DiscoveredTests> {
    // Test planning may inspect a secondary framework only to classify changed test files. Its
    // explicit policy is sufficient for that ownership check and avoids parsing an unrequested
    // runner config; requested runners still surface their prepared parse failures.
    let projects = prepared.requested_projects(runner).transpose()?.unwrap_or_else(|| {
        projects::explicit_policy_projects(root, config, runner)
    });
    discover_from_projects_from_visible(
        root,
        config,
        runner,
        projects,
        (runner == TestRunner::Vitest)
            .then(|| {
                prepared
                    .projects_if_prepared(TestRunner::Playwright)
                    .unwrap_or_else(|| {
                        projects::explicit_policy_projects(root, config, TestRunner::Playwright)
                    })
            }),
        visible_paths,
        tsconfig,
    )
}

pub fn discovered_test_globs(root: &Path, config: &NoMistakesConfig, runner: TestRunner) -> Result<Option<Vec<String>>> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, root, &visible_paths)?;
    discovered_test_globs_from_visible(root, config, runner, &visible_paths, &tsconfig)
}

#[doc(hidden)]
pub fn discovered_test_globs_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> Result<Option<Vec<String>>> {
    let discovered = discover_tests_from_visible(root, config, runner, visible_paths, tsconfig)?;
    if discovered.tests.is_empty() {
        return Ok(None);
    }
    Ok(Some(discovered.tests.iter().map(|path| {
        literal_path_glob(&crate::codebase::ts_source::relative_slash_path(root, path))
    }).collect()))
}

pub fn project_filters(root: &Path, config: &NoMistakesConfig) -> Vec<(TestRunner, ProjectTestFilter)> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let tsconfig = resolve_tsconfig_lossy(root, &visible_paths);
    project_filters_from_visible(root, config, &visible_paths, &tsconfig)
}

#[doc(hidden)]
pub fn project_filters_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> Vec<(TestRunner, ProjectTestFilter)> {
    [TestRunner::Dotnet, TestRunner::Vitest, TestRunner::Playwright, TestRunner::Swift]
        .into_iter()
        .flat_map(|runner| {
            projects::runner_projects_lossy_from_visible(root, config, runner, visible_paths, tsconfig)
                .into_iter()
                .filter_map(ProjectTestFilter::from_project)
                .map(move |filter| (runner, filter))
        })
        .collect()
}

pub(crate) fn named_project_filters(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    names: &[String],
) -> Vec<ProjectTestFilter> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let tsconfig = resolve_tsconfig_lossy(root, &visible_paths);
    named_project_filters_from_visible(root, config, runner, names, &visible_paths, &tsconfig)
}

#[doc(hidden)]
pub(crate) fn named_project_filters_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    names: &[String],
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> Vec<ProjectTestFilter> {
    projects::runner_projects_lossy_from_visible(root, config, runner, visible_paths, tsconfig)
        .into_iter()
        .filter(|project| project.policy_name.as_ref().is_some_and(|name| names.contains(name)))
        .filter_map(ProjectTestFilter::from_project)
        .collect()
}
