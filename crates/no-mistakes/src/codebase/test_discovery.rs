mod dotnet_projects;
mod filters;
mod ownership;
mod projects;
mod reserved;
mod swift_projects;
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
use ownership::owning_projects;
pub use targets::TestExecutionTarget;
pub use types::{DiscoveredTests, PreparedRunnerProject, TestRunner};
include!("test_discovery/preparation_plan.rs");
include!("test_discovery/prepared.rs");
include!("test_discovery/prepared_catalog.rs");
include!("test_discovery/prepared_vitest_resolution.rs");
include!("test_discovery/prepared_vitest_reparse.rs");
include!("test_discovery/prepared_vitest_setup.rs");
include!("test_discovery/api.rs");

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

fn discover_from_projects_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    projects: Vec<ConfigProject>,
    prepared_reserved_projects: Option<Vec<ConfigProject>>,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> Result<DiscoveredTests> {
    let files = crate::codebase::ts_source::discover_files_from_visible(
        root,
        &config.filesystem.skip_directories,
        visible_paths,
    );
    let mut tests = BTreeSet::new();
    let mut targets_by_path: BTreeMap<PathBuf, BTreeSet<TestExecutionTarget>> = BTreeMap::new();

    let compiled = projects
        .iter()
        .map(|project| Ok((project, ProjectTestFilter::from_project_ref(project)?)))
        .collect::<Result<Vec<_>>>()?;
    let mut project_scoped_paths = BTreeSet::new();
    for path in &files {
        let rel = crate::codebase::ts_source::relative_slash_path(root, path);
        let mut matched: Vec<&ConfigProject> = Vec::new();
        for (project, filter) in &compiled {
            if !filter.includes(&rel) {
                continue;
            }
            project_scoped_paths.insert(path.clone());
            if filter.excludes(&rel) {
                continue;
            }
            matched.push(project);
        }
        let matched_targets: BTreeSet<TestExecutionTarget> = owning_projects(&matched)
            .into_iter()
            .map(|project| {
                targets::target_for(
                    runner,
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
        root,
        config,
        runner,
        ProjectDiscoveryState {
            files,
            tests,
            targets_by_path,
            project_scoped_paths,
        },
        prepared_reserved_projects,
        visible_paths,
        tsconfig,
    ))
}

fn resolve_tsconfig_lossy(
    root: &Path,
    visible_paths: &[PathBuf],
) -> crate::codebase::ts_resolver::TsConfig {
    crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, root, visible_paths)
        .unwrap_or_else(|_| crate::codebase::ts_resolver::TsConfig {
            dir: root.to_path_buf(),
            paths: Vec::new(),
            paths_dir: root.to_path_buf(),
            base_url: None,
        })
}

struct ProjectDiscoveryState {
    files: Vec<PathBuf>,
    tests: BTreeSet<PathBuf>,
    targets_by_path: BTreeMap<PathBuf, BTreeSet<TestExecutionTarget>>,
    project_scoped_paths: BTreeSet<PathBuf>,
}

fn discover_with_fallback(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    state: ProjectDiscoveryState,
    prepared_reserved_projects: Option<Vec<ConfigProject>>,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> DiscoveredTests {
    let ProjectDiscoveryState {
        files,
        mut tests,
        mut targets_by_path,
        project_scoped_paths,
    } = state;
    let runner_reserved_tests = reserved::runner_reserved_tests_from_visible(
        root,
        config,
        runner,
        &files,
        prepared_reserved_projects,
        visible_paths,
        tsconfig,
    );
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
                .insert(targets::target_for(runner, None, false, None, &rel));
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
