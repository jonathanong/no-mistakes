use crate::config::v2::schema::{NoMistakesConfig, StringOrList, TestProjectPolicy};
use crate::integration_tests::project_config::prefix_globs;
use crate::integration_tests::types::ConfigProject;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::types::TestRunner;

#[cfg(test)]
mod tests;

pub(super) fn runner_projects_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> Result<Vec<ConfigProject>> {
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(root, tsconfig.clone(), None);
    runner_projects_from_visible_with_catalog(root, config, runner, visible_paths, &catalog)
}

pub(super) fn runner_projects_from_visible_with_catalog(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    visible_paths: &[PathBuf],
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
) -> Result<Vec<ConfigProject>> {
    if runner == TestRunner::Dotnet {
        return super::dotnet_projects::dotnet_projects_from_visible(root, config, visible_paths);
    }
    if runner == TestRunner::Swift {
        return Ok(super::swift_projects::swift_projects_from_visible(
            root,
            config,
            visible_paths,
        ));
    }
    let (configs, policies) = runner_config(config, runner);
    let mut projects =
        crate::integration_tests::project_config::load_projects_from_visible_with_catalog(
            root,
            runner.framework(),
            configs,
            visible_paths,
            tsconfig_catalog,
        )?;
    apply_explicit_policy_projects(root, configs, policies, &mut projects);
    Ok(projects)
}

pub(super) fn runner_projects_lossy_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> Vec<ConfigProject> {
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(root, tsconfig.clone(), None);
    runner_projects_lossy_from_visible_with_catalog(root, config, runner, visible_paths, &catalog)
}

pub(super) fn runner_projects_lossy_from_visible_with_catalog(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    visible_paths: &[PathBuf],
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
) -> Vec<ConfigProject> {
    if runner == TestRunner::Dotnet {
        return super::dotnet_projects::dotnet_projects_lossy_from_visible(
            root,
            config,
            visible_paths,
        );
    }
    if runner == TestRunner::Swift {
        return super::swift_projects::swift_projects_from_visible(root, config, visible_paths);
    }
    let (configs, policies) = runner_config(config, runner);
    let mut projects =
        crate::integration_tests::project_config::load_projects_from_visible_with_catalog(
            root,
            runner.framework(),
            configs,
            visible_paths,
            tsconfig_catalog,
        )
        .unwrap_or_default();
    apply_explicit_policy_projects(root, configs, policies, &mut projects);
    projects
}

/// Build only the projects described directly by no-mistakes policy.
///
/// This deliberately does not load runner config files. It lets a
/// demand-driven Vitest request reserve explicitly configured Playwright tests
/// without preparing Playwright itself.
pub(super) fn explicit_policy_projects(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> Vec<ConfigProject> {
    let (configs, policies) = runner_config(config, runner);
    policies
        .iter()
        .filter_map(|(name, policy)| {
            configured_project(root, name, policy, single_config(configs), Vec::new())
        })
        .collect()
}

fn apply_explicit_policy_projects(
    root: &Path,
    configs: Option<&StringOrList>,
    policies: &BTreeMap<String, TestProjectPolicy>,
    projects: &mut Vec<ConfigProject>,
) {
    for (name, policy) in policies {
        let matching_configs = projects
            .iter()
            .filter(|candidate| candidate.policy_name.as_deref() == Some(name))
            .map(|candidate| (candidate.config.clone(), candidate.vitest_setup.clone()))
            .collect::<Vec<_>>();
        let configs = if matching_configs.is_empty() {
            vec![(single_config(configs), Vec::new())]
        } else {
            matching_configs
        };
        let configured_projects = configs
            .into_iter()
            .filter_map(|(config, vitest_setup)| {
                configured_project(root, name, policy, config, vitest_setup)
            })
            .collect::<Vec<_>>();
        if !configured_projects.is_empty() {
            projects.retain(|candidate| candidate.policy_name.as_deref() != Some(name));
            projects.extend(configured_projects);
        }
    }
}

fn single_config(configs: Option<&StringOrList>) -> Option<String> {
    let configs = configs?;
    let values = configs.values();
    if values.len() == 1 {
        values.into_iter().next()
    } else {
        None
    }
}

pub(super) fn runner_config(
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> (Option<&StringOrList>, &BTreeMap<String, TestProjectPolicy>) {
    match runner {
        TestRunner::Dotnet => unreachable!("dotnet projects are handled before runner_config"),
        TestRunner::Playwright => (
            config.tests.playwright.configs.as_ref(),
            &config.tests.playwright.projects,
        ),
        TestRunner::Vitest => (
            config.tests.vitest.configs.as_ref(),
            &config.tests.vitest.projects,
        ),
        TestRunner::Swift => (None, &config.tests.swift.projects),
    }
}

fn configured_project(
    root: &Path,
    project_name: &str,
    policy: &TestProjectPolicy,
    config: Option<String>,
    vitest_setup: Vec<crate::integration_tests::types::VitestSetupDependency>,
) -> Option<ConfigProject> {
    if policy.include.is_empty() {
        return None;
    }
    Some(ConfigProject {
        config,
        policy_name: Some(project_name.to_string()),
        runner_project_arg: Some(project_name.to_string()),
        // Explicit policies own their declared include/exclude universe, not
        // the parsed runner root. Setup fallback keeps its parsed resolution
        // base separately on each setup dependency.
        scope: None,
        include: prefix_globs(root, root, &policy.include),
        exclude: prefix_globs(root, root, &policy.exclude),
        vitest_setup,
    })
}
