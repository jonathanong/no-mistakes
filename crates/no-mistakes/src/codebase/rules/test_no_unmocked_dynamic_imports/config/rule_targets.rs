use crate::config::v2::NoMistakesConfig;
use crate::integration_tests::types::{ConfigProject, Framework};
use anyhow::Result;
use std::path::{Path, PathBuf};

mod path_patterns;
mod projects;
use super::build_globset;
use path_patterns::{append_project_globs, append_rule_globs};

pub(super) struct PreparedTargets {
    pub(super) includes: Vec<String>,
    pub(super) excludes: Vec<String>,
    playwright: Vec<ConfigProject>,
}

impl PreparedTargets {
    pub(super) fn playwright_setup_data(&self) -> Result<Vec<super::ConfigSetupData>> {
        self.playwright
            .iter()
            .cloned()
            .map(|project| {
                Ok(super::ConfigSetupData {
                    workspace: false,
                    filter: super::TestFilter {
                        alternatives: Vec::new(),
                        include: build_globset(&project.include)?,
                        include_regex: Vec::new(),
                        exclude: build_globset(&project.exclude)?,
                    },
                    setup_files: Vec::new(),
                    explicit_project: false,
                    runner: super::discovery::Runner::Playwright,
                    project_name: project.policy_name,
                })
            })
            .collect()
    }
}

pub(super) fn rule_test_project_globs(
    root: &Path,
    config: &NoMistakesConfig,
) -> Result<(Vec<String>, Vec<String>)> {
    let targets = prepare(root, config, None)?;
    Ok((targets.includes, targets.excludes))
}

pub(super) fn prepare(
    root: &Path,
    config: &NoMistakesConfig,
    prepared: Option<(&[PathBuf], &crate::codebase::check_facts::RunnerConfigFacts)>,
) -> Result<PreparedTargets> {
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
    let vitest_projects = projects::load(
        root,
        Framework::Vitest,
        config.tests.vitest.configs.as_ref(),
        &config.tests.vitest.projects,
        &vitest_project_names,
        prepared,
    );
    let vitest_projects = vitest_projects?;
    let playwright_projects =
        if prepared.is_some() && super::SelectedRunners::for_config(config).includes_all() {
            projects::load_all(
                root,
                Framework::Playwright,
                config.tests.playwright.configs.as_ref(),
                &config.tests.playwright.projects,
                prepared,
            )
        } else {
            projects::load(
                root,
                Framework::Playwright,
                config.tests.playwright.configs.as_ref(),
                &config.tests.playwright.projects,
                &playwright_project_names,
                prepared,
            )
        };
    let playwright_projects = playwright_projects?;
    for rule in rules {
        append_rule_globs(config, rule, &mut includes, &mut excludes);
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
            append_project_globs(config, rule, project_name, &mut includes, &mut excludes);
        }
    }
    includes.sort();
    includes.dedup();
    excludes.sort();
    excludes.dedup();
    Ok(PreparedTargets {
        includes,
        excludes,
        playwright: playwright_projects,
    })
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
    project.policy_name.as_deref() == Some(project_name)
        || (project.policy_name.is_none() && project_name == "default")
}
