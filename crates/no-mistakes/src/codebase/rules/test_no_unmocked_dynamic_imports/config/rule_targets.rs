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
    pub(super) executions: Vec<ExecutionGlobs>,
    playwright: Vec<ConfigProject>,
}

pub(super) struct ExecutionGlobs {
    pub(super) include: Vec<String>,
    pub(super) exclude: Vec<String>,
    pub(super) scope: Vec<String>,
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
                        required: None,
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
) -> Result<PreparedTargets> {
    prepare(root, config, None)
}

pub(super) fn prepare(
    root: &Path,
    config: &NoMistakesConfig,
    prepared: Option<(&[PathBuf], &crate::codebase::check_facts::RunnerConfigFacts)>,
) -> Result<PreparedTargets> {
    let mut includes = Vec::new();
    let mut excludes = Vec::new();
    let mut executions = Vec::new();
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
        let has_test_targets = !rule.tests.vitest.is_empty() || !rule.tests.playwright.is_empty();
        let mut rule_includes = Vec::new();
        append_rule_globs(config, rule, &mut rule_includes, &mut excludes);
        let mut scope = Vec::new();
        if has_test_targets {
            if rule.projects.is_empty() {
                scope.extend(rule.include.iter().cloned());
            } else {
                for project_name in &rule.projects {
                    append_project_globs(config, rule, project_name, &mut scope, &mut excludes);
                }
                if scope.is_empty() {
                    scope.extend(rule.include.iter().cloned());
                }
            }
        } else {
            includes.extend(rule_includes);
        }
        for project_name in &rule.tests.vitest {
            append_test_project_globs(
                Framework::Vitest,
                project_name,
                &vitest_projects,
                &scope,
                &mut executions,
            )?;
        }
        for project_name in &rule.tests.playwright {
            append_test_project_globs(
                Framework::Playwright,
                project_name,
                &playwright_projects,
                &scope,
                &mut executions,
            )?;
        }
        if !has_test_targets {
            for project_name in &rule.projects {
                append_project_globs(config, rule, project_name, &mut includes, &mut excludes);
            }
        }
    }
    includes.sort();
    includes.dedup();
    excludes.sort();
    excludes.dedup();
    Ok(PreparedTargets {
        includes,
        excludes,
        executions,
        playwright: playwright_projects,
    })
}

fn append_test_project_globs(
    framework: Framework,
    project_name: &str,
    projects: &[ConfigProject],
    scope: &[String],
    executions: &mut Vec<ExecutionGlobs>,
) -> Result<()> {
    let mut matched = false;
    for project in projects
        .iter()
        .filter(|project| test_project_matches(project, project_name))
    {
        matched = true;
        // An exclusion belongs to this execution only. Another selected
        // project can still execute the same file.
        executions.push(ExecutionGlobs {
            include: project.include.clone(),
            exclude: project.exclude.clone(),
            scope: scope.to_vec(),
        });
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
