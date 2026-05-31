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

    if projects.is_empty() {
        return Ok(discover_with_fallback(
            root,
            runner,
            files,
            tests,
            targets_by_path,
        ));
    }

    let compiled = projects
        .iter()
        .map(|project| Ok((project, ProjectTestFilter::from_project_ref(project)?)))
        .collect::<Result<Vec<_>>>()?;
    for path in files {
        let rel = crate::codebase::ts_source::relative_slash_path(root, &path);
        let mut matched_targets = BTreeSet::new();
        for (project, filter) in &compiled {
            if !filter.is_match(&rel) {
                continue;
            }
            matched_targets.insert(targets::target_for(
                runner,
                project.config.as_deref(),
                project.name.as_deref(),
                &rel,
            ));
        }
        if !matched_targets.is_empty() {
            tests.insert(path.clone());
            targets_by_path
                .entry(path)
                .or_default()
                .extend(matched_targets);
        }
    }

    Ok(to_discovered(tests, targets_by_path, false))
}

fn discover_with_fallback(
    root: &Path,
    runner: TestRunner,
    files: Vec<PathBuf>,
    mut tests: BTreeSet<PathBuf>,
    mut targets_by_path: BTreeMap<PathBuf, BTreeSet<TestExecutionTarget>>,
) -> DiscoveredTests {
    for path in files {
        let rel = crate::codebase::ts_source::relative_slash_path(root, &path);
        if filters::fallback_runner_match(runner, &rel) {
            tests.insert(path.clone());
            targets_by_path
                .entry(path)
                .or_default()
                .insert(targets::target_for(runner, None, None, &rel));
        }
    }
    to_discovered(tests, targets_by_path, true)
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
