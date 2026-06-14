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
    if runner == TestRunner::Swift {
        return Ok(swift_projects(root, config));
    }
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
    if runner == TestRunner::Swift {
        return swift_projects(root, config);
    }
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
            .filter(|candidate| candidate.policy_name.as_deref() == Some(name))
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
        TestRunner::Swift => (None, &config.tests.swift.projects),
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
        policy_name: Some(project_name.to_string()),
        runner_project_arg: Some(project_name.to_string()),
        scope: None,
        include: prefix_globs(root, root, &policy.include),
        exclude: prefix_globs(root, root, &policy.exclude),
    })
}

fn swift_projects(root: &Path, config: &NoMistakesConfig) -> Vec<ConfigProject> {
    let mut projects = Vec::new();
    for package in &config.tests.swift.packages {
        let package_root = root.join(package);
        let package_slash = package.trim_end_matches('/');
        let package_projects = swift_test_targets_from_package(root, &package_root, package_slash);
        if package_projects.is_empty() {
            projects.push(ConfigProject {
                config: Some(package_slash.to_string()),
                policy_name: Some(package_slash.to_string()),
                runner_project_arg: None,
                scope: Some(package_slash.to_string()),
                include: vec![format!("{package_slash}/Tests/**/*.swift")],
                exclude: Vec::new(),
            });
        } else {
            projects.extend(package_projects);
        }
    }
    for (name, policy) in &config.tests.swift.projects {
        if let Some(project) =
            configured_swift_project(root, name, policy, &config.tests.swift.packages)
        {
            projects.retain(|candidate| candidate.policy_name.as_deref() != Some(name));
            projects.push(project);
        }
    }
    projects
}

fn configured_swift_project(
    root: &Path,
    project_name: &str,
    policy: &TestProjectPolicy,
    packages: &[String],
) -> Option<ConfigProject> {
    if policy.include.is_empty() {
        return None;
    }
    Some(ConfigProject {
        config: swift_package_for_policy(project_name, policy, packages),
        policy_name: Some(project_name.to_string()),
        runner_project_arg: None,
        scope: None,
        include: prefix_globs(root, root, &policy.include),
        exclude: prefix_globs(root, root, &policy.exclude),
    })
}

fn swift_package_for_policy(
    project_name: &str,
    policy: &TestProjectPolicy,
    packages: &[String],
) -> Option<String> {
    for package in packages {
        let package = package.trim_end_matches('/');
        if project_name == package || policy.include.iter().any(|glob| glob.starts_with(package)) {
            return Some(package.to_string());
        }
    }
    packages
        .first()
        .map(|package| package.trim_end_matches('/').to_string())
}

fn swift_test_targets_from_package(
    root: &Path,
    package_root: &Path,
    package: &str,
) -> Vec<ConfigProject> {
    let manifest = package_root.join("Package.swift");
    let Ok(source) = std::fs::read_to_string(manifest) else {
        return Vec::new();
    };
    let mut projects = Vec::new();
    for target in crate::codebase::swift::extract_test_target_names(&source) {
        let include = vec![format!("{package}/Tests/{target}/**/*.swift")];
        projects.push(ConfigProject {
            config: Some(package.to_string()),
            policy_name: Some(target.clone()),
            runner_project_arg: Some(target),
            scope: Some(format!("{package}/Tests")),
            include: prefix_globs(root, root, &include),
            exclude: Vec::new(),
        });
    }
    projects
}
