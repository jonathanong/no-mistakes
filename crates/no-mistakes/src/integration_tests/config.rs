use super::project_config::prefix_globs;
use super::runner_config::{ParsedRunnerConfigs, PreparedIntegrationRunnerConfigs};
use super::types::{ConfigProject, EffectiveIntegrationPolicy, Framework, Suite};
use crate::config::v2::schema::{NoMistakesConfig, TestProjectPolicy};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) fn validate_config(config: &NoMistakesConfig) -> Result<()> {
    for (framework, projects) in [
        ("playwright", &config.tests.playwright.projects),
        ("vitest", &config.tests.vitest.projects),
    ] {
        for (project, policy) in projects {
            if policy.include.is_empty() && !policy.exclude.is_empty() {
                anyhow::bail!(
                    "tests.{}.projects.{}.exclude requires include",
                    framework,
                    project
                );
            }
            for (suite, integrations) in &policy.integration_suites {
                if integrations.is_empty() {
                    anyhow::bail!(
                        "tests.{}.projects.{}.integration_suites.{} must contain at least one integration",
                        framework,
                        project,
                        suite
                    );
                }
            }
        }
    }
    Ok(())
}

pub(super) fn configured_suites_from_runner_configs(
    root: &Path,
    config: &NoMistakesConfig,
    runner_configs: &PreparedIntegrationRunnerConfigs,
    parsed: &ParsedRunnerConfigs,
) -> Result<Vec<Suite>> {
    let mut suites = Vec::new();
    let playwright_projects = parsed.projects_for(runner_configs, Framework::Playwright)?;
    suites.extend(suites_for_framework(
        root,
        Framework::Playwright,
        &config.tests.playwright.projects,
        &playwright_projects,
    )?);
    let vitest_projects = parsed.projects_for(runner_configs, Framework::Vitest)?;
    suites.extend(suites_for_framework(
        root,
        Framework::Vitest,
        &config.tests.vitest.projects,
        &vitest_projects,
    )?);
    Ok(suites)
}

fn suites_for_framework(
    root: &Path,
    framework: Framework,
    policies: &BTreeMap<String, TestProjectPolicy>,
    projects: &[ConfigProject],
) -> Result<Vec<Suite>> {
    if policies
        .values()
        .all(|policy| policy.integration_suites.is_empty())
    {
        return Ok(Vec::new());
    }

    let mut suites = Vec::new();
    for (project_name, policy) in policies {
        if policy.integration_suites.is_empty() {
            continue;
        }
        let configured_project = configured_project(root, project_name, policy);
        let project = match configured_project.as_ref() {
            Some(project) => project,
            None => exact_project(framework, project_name, projects)?,
        };
        let allowed_integrations = policy
            .integration_suites
            .values()
            .flatten()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        suites.push(Suite {
            framework,
            name: policy_suite_name(project_name, policy),
            include: project.include.clone(),
            exclude: project.exclude.clone(),
            policy: EffectiveIntegrationPolicy::AllowedIntegrations {
                integrations: allowed_integrations,
            },
        });
    }
    Ok(suites)
}

pub(crate) fn configured_project(
    root: &Path,
    project_name: &str,
    policy: &TestProjectPolicy,
) -> Option<ConfigProject> {
    if policy.include.is_empty() {
        return None;
    }
    Some(ConfigProject {
        config: None,
        workspace: false,
        policy_name: Some(project_name.to_string()),
        runner_project_arg: Some(project_name.to_string()),
        scope: None,
        include: prefix_globs(root, root, &policy.include),
        exclude: prefix_globs(root, root, &policy.exclude),
        vitest_setup: Vec::new(),
    })
}

fn policy_suite_name(project_name: &str, policy: &TestProjectPolicy) -> String {
    let suffix = policy
        .integration_suites
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .join("+");
    format!("{project_name}.{suffix}")
}

fn exact_project<'a>(
    framework: Framework,
    project_name: &str,
    projects: &'a [ConfigProject],
) -> Result<&'a ConfigProject> {
    let matches = projects
        .iter()
        .filter(|project| project.policy_name.as_deref() == Some(project_name))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [project] => Ok(*project),
        [] => Err(anyhow::anyhow!(
            "{} integration policy references unknown project {}",
            framework.as_str(),
            project_name
        )),
        matches => Err(anyhow::anyhow!(
            "{} integration policy references ambiguous project {} ({} matches)",
            framework.as_str(),
            project_name,
            matches.len()
        )),
    }
}
