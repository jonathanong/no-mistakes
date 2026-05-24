use crate::config::v2::schema::{StringOrList, TestProjectPolicy};
use crate::config::v2::NoMistakesConfig;
use crate::integration_tests::project_config::load_projects;
use crate::integration_tests::types::{ConfigProject, Framework};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) fn rule_test_project_globs(
    root: &Path,
    config: &NoMistakesConfig,
) -> Result<(Vec<String>, Vec<String>)> {
    let mut includes = Vec::new();
    let mut excludes = Vec::new();
    let rules = config.rule_applications(super::super::RULE_ID);
    let vitest_project_names = rules
        .iter()
        .flat_map(|rule| rule.tests.vitest.iter())
        .collect::<Vec<_>>();
    let playwright_project_names = rules
        .iter()
        .flat_map(|rule| rule.tests.playwright.iter())
        .collect::<Vec<_>>();
    let vitest_projects = load_target_projects(
        root,
        Framework::Vitest,
        config.tests.vitest.configs.as_ref(),
        &config.tests.vitest.projects,
        &vitest_project_names,
    )?;
    let playwright_projects = load_target_projects(
        root,
        Framework::Playwright,
        config.tests.playwright.configs.as_ref(),
        &config.tests.playwright.projects,
        &playwright_project_names,
    )?;
    for rule in rules {
        for project_name in &rule.tests.vitest {
            append_test_project_globs(
                Framework::Vitest,
                project_name,
                &vitest_projects,
                &mut includes,
                &mut excludes,
            )?;
        }
        for project_name in &rule.tests.playwright {
            append_test_project_globs(
                Framework::Playwright,
                project_name,
                &playwright_projects,
                &mut includes,
                &mut excludes,
            )?;
        }
        for project_name in &rule.projects {
            append_project_includes(config, project_name, &mut includes);
        }
    }
    includes.sort();
    includes.dedup();
    excludes.sort();
    excludes.dedup();
    Ok((includes, excludes))
}

fn load_target_projects(
    root: &Path,
    framework: Framework,
    configs: Option<&StringOrList>,
    policies: &BTreeMap<String, TestProjectPolicy>,
    project_names: &[&String],
) -> Result<Vec<ConfigProject>> {
    if project_names.is_empty() {
        return Ok(Vec::new());
    }
    let mut unresolved = Vec::new();
    let mut projects = Vec::new();
    for project_name in project_names {
        if let Some(project) = policies.get(*project_name).and_then(|policy| {
            crate::integration_tests::config::configured_project(root, project_name, policy)
        }) {
            projects.push(project);
        } else {
            unresolved.push(*project_name);
        }
    }
    if !unresolved.is_empty() {
        let unresolved_names = unresolved
            .iter()
            .map(|name| name.as_str())
            .collect::<BTreeSet<_>>();
        projects.extend(
            load_projects(root, framework, configs)?
                .into_iter()
                .filter(|project| {
                    unresolved_names
                        .iter()
                        .any(|name| test_project_matches(project, name))
                }),
        );
    }
    Ok(projects)
}

fn append_test_project_globs(
    framework: Framework,
    project_name: &str,
    projects: &[ConfigProject],
    includes: &mut Vec<String>,
    excludes: &mut Vec<String>,
) -> Result<()> {
    let mut matched = false;
    for project in projects
        .iter()
        .filter(|project| test_project_matches(project, project_name))
    {
        matched = true;
        includes.extend(project.include.clone());
        excludes.extend(project.exclude.clone());
    }
    if !matched {
        anyhow::bail!(
            "test-no-unmocked-dynamic-imports references unknown {} project {project_name}",
            framework.as_str()
        );
    }
    Ok(())
}

fn test_project_matches(project: &ConfigProject, project_name: &str) -> bool {
    project.name.as_deref() == Some(project_name)
        || (project.name.is_none() && project_name == "default")
}

fn append_project_includes(
    config: &NoMistakesConfig,
    project_name: &str,
    includes: &mut Vec<String>,
) {
    let Some(project) = config.projects.get(project_name) else {
        return;
    };
    let root = project.root.as_deref().unwrap_or(".").trim_matches('/');
    let project_includes = if project.include.is_empty() {
        vec!["**".to_string()]
    } else {
        project.include.clone()
    };
    for include in project_includes {
        if root.is_empty() || root == "." {
            includes.push(include);
        } else {
            includes.push(format!(
                "{}/{}",
                root.trim_start_matches("./"),
                include.trim_start_matches("./")
            ));
        }
    }
}
