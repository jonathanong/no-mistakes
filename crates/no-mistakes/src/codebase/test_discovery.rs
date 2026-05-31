mod filters;
mod projects;
mod targets;
mod types;

#[cfg(test)]
mod tests;

use crate::config::v2::schema::NoMistakesConfig;
use crate::integration_tests::types::ConfigProject;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) use filters::fallback_runner_match;
pub use filters::{fallback_test_path, ProjectTestFilter};
pub use targets::TestExecutionTarget;
pub use types::{DiscoveredTests, TestRunner};

pub fn discover_tests(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> Result<DiscoveredTests> {
    let projects = projects::runner_projects(root, config, runner)?;
    discover_from_projects(root, config, runner, projects)
}

pub fn discovered_test_globs(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> Result<Option<Vec<String>>> {
    let discovered = discover_tests(root, config, runner)?;
    if discovered.tests.is_empty() {
        return Ok(None);
    }
    Ok(Some(
        discovered
            .tests
            .iter()
            .map(|path| {
                literal_path_glob(&crate::codebase::ts_source::relative_slash_path(root, path))
            })
            .collect(),
    ))
}

pub fn project_filters(
    root: &Path,
    config: &NoMistakesConfig,
) -> Vec<(TestRunner, ProjectTestFilter)> {
    let mut filters = Vec::new();
    for runner in [TestRunner::Vitest, TestRunner::Playwright] {
        let projects = projects::runner_projects_lossy(root, config, runner);
        filters.extend(
            projects
                .into_iter()
                .filter_map(ProjectTestFilter::from_project)
                .map(|filter| (runner, filter)),
        );
    }
    filters
}

pub fn literal_path_glob(path: &str) -> String {
    let mut escaped = String::with_capacity(path.len());
    for ch in path.chars() {
        if matches!(ch, '*' | '?' | '[' | ']' | '{' | '}' | '\\') {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

fn discover_from_projects(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    projects: Vec<ConfigProject>,
) -> Result<DiscoveredTests> {
    let files =
        crate::codebase::ts_source::discover_files(root, &config.filesystem.skip_directories);
    let mut tests = BTreeSet::new();
    let mut targets_by_path: BTreeMap<PathBuf, BTreeSet<TestExecutionTarget>> = BTreeMap::new();

    let compiled = projects
        .iter()
        .map(|project| Ok((project, ProjectTestFilter::from_project_ref(project)?)))
        .collect::<Result<Vec<_>>>()?;
    let mut project_scoped_paths = BTreeSet::new();
    for path in &files {
        let rel = crate::codebase::ts_source::relative_slash_path(root, path);
        let mut matched_targets = BTreeSet::new();
        for (project, filter) in &compiled {
            if !filter.includes(&rel) {
                continue;
            }
            project_scoped_paths.insert(path.clone());
            if filter.excludes(&rel) {
                continue;
            }
            matched_targets.insert(targets::target_for(
                runner,
                project.config.as_deref(),
                project.target_project.as_deref(),
                &rel,
            ));
        }
        if !matched_targets.is_empty() {
            tests.insert(path.clone());
            targets_by_path
                .entry(path.clone())
                .or_default()
                .extend(matched_targets);
        }
    }

    Ok(discover_with_fallback(
        root,
        config,
        runner,
        files,
        tests,
        targets_by_path,
        project_scoped_paths,
    ))
}

fn discover_with_fallback(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    files: Vec<PathBuf>,
    mut tests: BTreeSet<PathBuf>,
    mut targets_by_path: BTreeMap<PathBuf, BTreeSet<TestExecutionTarget>>,
    project_scoped_paths: BTreeSet<PathBuf>,
) -> DiscoveredTests {
    let runner_reserved_tests = runner_reserved_tests(root, config, runner, &files);
    let mut used_fallback = false;
    for path in files {
        if tests.contains(&path)
            || project_scoped_paths.contains(&path)
            || runner_reserved_tests.contains(&path)
        {
            continue;
        }
        let rel = crate::codebase::ts_source::relative_slash_path(root, &path);
        if filters::fallback_runner_match(runner, &rel) {
            used_fallback = true;
            tests.insert(path.clone());
            targets_by_path
                .entry(path)
                .or_default()
                .insert(targets::target_for(runner, None, None, &rel));
        }
    }
    to_discovered(tests, targets_by_path, used_fallback)
}

fn runner_reserved_tests(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    files: &[PathBuf],
) -> BTreeSet<PathBuf> {
    if runner != TestRunner::Vitest {
        return BTreeSet::new();
    }
    let playwright_projects = projects::runner_projects_lossy(root, config, TestRunner::Playwright);
    if playwright_projects.is_empty() {
        return BTreeSet::new();
    }
    let playwright_filters = playwright_projects
        .into_iter()
        .filter_map(ProjectTestFilter::from_project)
        .collect::<Vec<_>>();
    files
        .iter()
        .filter(|path| {
            let rel = crate::codebase::ts_source::relative_slash_path(root, path);
            playwright_filters
                .iter()
                .any(|filter| filter.is_match(&rel))
        })
        .cloned()
        .collect()
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
