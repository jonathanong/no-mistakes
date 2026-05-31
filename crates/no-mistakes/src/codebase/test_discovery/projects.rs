use crate::config::v2::schema::{NoMistakesConfig, StringOrList, TestProjectPolicy};
use crate::integration_tests::project_config::prefix_globs;
use crate::integration_tests::types::ConfigProject;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

use super::types::TestRunner;

pub(super) fn runner_projects(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> Result<Vec<ConfigProject>> {
    let (configs, policies) = runner_config(config, runner);
    let mut projects =
        crate::integration_tests::project_config::load_projects(root, runner.framework(), configs)?;
    apply_explicit_policy_projects(root, policies, &mut projects);
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
    apply_explicit_policy_projects(root, policies, &mut projects);
    projects
}

fn apply_explicit_policy_projects(
    root: &Path,
    policies: &BTreeMap<String, TestProjectPolicy>,
    projects: &mut Vec<ConfigProject>,
) {
    for (name, policy) in policies {
        let config = projects
            .iter()
            .find(|candidate| candidate.name.as_deref() == Some(name))
            .and_then(|candidate| candidate.config.clone());
        if let Some(project) = configured_project(root, name, policy, config) {
            projects.retain(|candidate| candidate.name.as_deref() != Some(name));
            projects.push(project);
        }
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
        include: prefix_globs(root, root, &policy.include),
        exclude: prefix_globs(root, root, &policy.exclude),
    })
}
