use crate::config::v2::schema::{NoMistakesConfig, StringOrList, TestProjectPolicy};
use crate::integration_tests::project_config::prefix_globs;
use crate::integration_tests::types::ConfigProject;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

use super::types::TestRunner;

#[cfg(test)]
mod tests;

pub(super) fn runner_projects(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> Result<Vec<ConfigProject>> {
    let (configs, policies) = runner_config(config, runner);
    let mut projects =
        crate::integration_tests::project_config::load_projects(root, runner.framework(), configs)?;
    apply_explicit_policy_projects(root, configs, policies, &mut projects);
    Ok(projects)
}

pub(super) fn runner_projects_lossy(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> Vec<ConfigProject> {
    let (configs, policies) = runner_config(config, runner);
    let mut projects =
        crate::integration_tests::project_config::load_projects(root, runner.framework(), configs)
            .unwrap_or_default();
    apply_explicit_policy_projects(root, configs, policies, &mut projects);
    projects
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
            .filter(|candidate| candidate.name.as_deref() == Some(name))
            .map(|candidate| candidate.config.clone())
            .collect::<Vec<_>>();
        let configs = if matching_configs.is_empty() {
            vec![single_config(configs)]
        } else {
            matching_configs
        };
        let configured_projects = configs
            .into_iter()
            .filter_map(|config| configured_project(root, name, policy, config))
            .collect::<Vec<_>>();
        if !configured_projects.is_empty() {
            projects.retain(|candidate| candidate.name.as_deref() != Some(name));
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

fn runner_config(
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> (Option<&StringOrList>, &BTreeMap<String, TestProjectPolicy>) {
    match runner {
        TestRunner::Playwright => (
            config.tests.playwright.configs.as_ref(),
            &config.tests.playwright.projects,
        ),
        TestRunner::Vitest => (
            config.tests.vitest.configs.as_ref(),
            &config.tests.vitest.projects,
        ),
    }
}

fn configured_project(
    root: &Path,
    project_name: &str,
    policy: &TestProjectPolicy,
    config: Option<String>,
) -> Option<ConfigProject> {
    if policy.include.is_empty() {
        return None;
    }
    Some(ConfigProject {
        config,
        name: Some(project_name.to_string()),
        target_project: Some(project_name.to_string()),
        include: prefix_globs(root, root, &policy.include),
        exclude: prefix_globs(root, root, &policy.exclude),
    })
}
