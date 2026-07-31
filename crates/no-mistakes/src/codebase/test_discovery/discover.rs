use crate::integration_tests::types::ConfigProject;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
struct DiscoveryRequest<'a> {
    root: &'a Path,
    config: &'a NoMistakesConfig,
    runner: TestRunner,
    visible_paths: &'a [PathBuf],
    tsconfig: &'a crate::codebase::ts_resolver::TsConfig,
}

impl<'a> DiscoveryRequest<'a> {
    fn new(
        root: &'a Path,
        config: &'a NoMistakesConfig,
        runner: TestRunner,
        visible_paths: &'a [PathBuf],
        tsconfig: &'a crate::codebase::ts_resolver::TsConfig,
    ) -> Self {
        Self {
            root,
            config,
            runner,
            visible_paths,
            tsconfig,
        }
    }
}

fn discover_from_projects_from_visible(
    request: DiscoveryRequest<'_>,
    projects: Vec<ConfigProject>,
    prepared_reserved_projects: Option<Vec<ConfigProject>>,
) -> Result<DiscoveredTests> {
    let files = crate::codebase::ts_source::discover_files_from_visible(
        request.root,
        &request.config.filesystem.skip_directories,
        request.visible_paths,
    );
    discover_from_projects_from_files(request, projects, prepared_reserved_projects, &files)
}

fn discover_from_projects_from_files(
    request: DiscoveryRequest<'_>,
    projects: Vec<ConfigProject>,
    prepared_reserved_projects: Option<Vec<ConfigProject>>,
    files: &[PathBuf],
) -> Result<DiscoveredTests> {
    let mut tests = BTreeSet::new();
    let mut targets_by_path: BTreeMap<PathBuf, BTreeSet<TestExecutionTarget>> = BTreeMap::new();
    let authoritative_projects = matches!(
        request.runner,
        TestRunner::Vitest | TestRunner::Playwright
    ) && !projects.is_empty();
    let compiled = projects
        .iter()
        .map(|project| Ok((project, ProjectTestFilter::from_project_ref(project)?)))
        .collect::<Result<Vec<_>>>()?;
    let mut project_scoped_paths = BTreeSet::new();
    for path in files {
        let rel = crate::codebase::ts_source::relative_slash_path(request.root, path);
        let mut matched: Vec<&ConfigProject> = Vec::new();
        for (project, filter) in &compiled {
            if !filter.includes(&rel) {
                continue;
            }
            project_scoped_paths.insert(path.clone());
            if !filter.excludes(&rel) {
                matched.push(project);
            }
        }
        let matched_targets: BTreeSet<TestExecutionTarget> = owning_projects(&matched)
            .into_iter()
            .map(|project| {
                targets::target_for(
                    request.runner,
                    project.config.as_deref(),
                    project.workspace,
                    project.runner_project_arg.as_deref(),
                    &rel,
                )
            })
            .collect();
        if !matched_targets.is_empty() {
            tests.insert(path.clone());
            targets_by_path
                .entry(path.clone())
                .or_default()
                .extend(matched_targets);
        }
    }
    Ok(discover_with_fallback(
        request,
        ProjectDiscoveryState {
            files,
            tests,
            targets_by_path,
            project_scoped_paths,
            authoritative_projects,
        },
        prepared_reserved_projects,
    ))
}

struct ProjectDiscoveryState<'a> {
    files: &'a [PathBuf],
    tests: BTreeSet<PathBuf>,
    targets_by_path: BTreeMap<PathBuf, BTreeSet<TestExecutionTarget>>,
    project_scoped_paths: BTreeSet<PathBuf>,
    authoritative_projects: bool,
}

fn discover_with_fallback(
    request: DiscoveryRequest<'_>,
    state: ProjectDiscoveryState<'_>,
    prepared_reserved_projects: Option<Vec<ConfigProject>>,
) -> DiscoveredTests {
    let ProjectDiscoveryState {
        files,
        mut tests,
        mut targets_by_path,
        project_scoped_paths,
        authoritative_projects,
    } = state;
    if authoritative_projects {
        return to_discovered(tests, targets_by_path, false);
    }
    let runner_reserved_tests = reserved::runner_reserved_tests_from_visible(
        request.root,
        request.config,
        request.runner,
        files,
        prepared_reserved_projects,
        request.visible_paths,
        request.tsconfig,
    );
    let mut used_fallback = false;
    for path in files {
        if tests.contains(path)
            || project_scoped_paths.contains(path)
            || runner_reserved_tests.contains(path)
        {
            continue;
        }
        let rel = crate::codebase::ts_source::relative_slash_path(request.root, path);
        if filters::fallback_runner_match(request.runner, &rel) {
            used_fallback = true;
            tests.insert(path.clone());
            targets_by_path
                .entry(path.clone())
                .or_default()
                .insert(targets::target_for(
                    request.runner,
                    None,
                    false,
                    None,
                    &rel,
                ));
        }
    }
    to_discovered(tests, targets_by_path, used_fallback)
}

fn to_discovered(
    tests: BTreeSet<PathBuf>,
    targets_by_path: BTreeMap<PathBuf, BTreeSet<TestExecutionTarget>>,
    used_fallback: bool,
) -> DiscoveredTests {
    DiscoveredTests {
        tests: tests.into_iter().collect(),
        targets_by_path: targets_by_path
            .into_iter()
            .map(|(path, targets)| (path, targets.into_iter().collect()))
            .collect(),
        used_fallback,
    }
}
