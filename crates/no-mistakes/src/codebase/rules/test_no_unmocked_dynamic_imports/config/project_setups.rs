use super::{build_globset, ConfigSetupData, TestFilter};
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(in super::super) fn explicit_project_setup_data(
    root: &Path,
    config: &NoMistakesConfig,
    visible_files: &crate::fx::PathSet,
) -> Result<Vec<ConfigSetupData>> {
    let mut data = Vec::new();
    let rules = config.rule_applications(super::super::RULE_ID);
    let all_projects = all_runner_projects(config);
    let selected = rules
        .iter()
        .flat_map(|rule| rule.tests.vitest.iter().map(String::as_str))
        .collect::<std::collections::HashSet<_>>();
    for (name, project) in &config.tests.vitest.projects {
        if !all_projects && !selected.contains(name.as_str()) {
            continue;
        }
        if project.include.is_empty() && project.setup_files.is_empty() {
            continue;
        }
        project.validate_vitest_setup_files(name)?;
        let filter = TestFilter {
            alternatives: Vec::new(),
            include: build_globset(&project.include)?,
            include_regex: Vec::new(),
            exclude: build_globset(&project.exclude)?,
            required: None,
        };
        let mut setup_files = Vec::new();
        for setup in &project.setup_files {
            let relative = Path::new(setup);
            let path = crate::codebase::ts_resolver::normalize_path(&root.join(relative));
            anyhow::ensure!(
                visible_files.contains(&path),
                "tests.vitest.projects.{name}.setup_files path {setup:?} is missing from the analysis file inventory"
            );
            setup_files.push(path);
        }
        let mut seen = std::collections::HashSet::new();
        setup_files.retain(|path| seen.insert(path.clone()));
        data.push(ConfigSetupData {
            workspace: false,
            filter,
            setup_files,
            explicit_project: true,
            runner: super::discovery::Runner::Vitest,
            project_name: Some(name.clone()),
        });
    }
    Ok(data)
}

#[derive(Clone, Copy)]
pub(in super::super) struct SelectedRunners {
    vitest: bool,
    jest: bool,
}

impl SelectedRunners {
    pub(super) fn includes_all(self) -> bool {
        self.jest
    }

    pub(crate) fn includes_vitest(self) -> bool {
        self.vitest
    }

    pub(super) fn includes(self, runner: super::discovery::Runner) -> bool {
        match runner {
            super::discovery::Runner::Vitest => self.vitest,
            super::discovery::Runner::Jest => self.jest,
            super::discovery::Runner::Playwright => true,
        }
    }

    pub(in super::super) fn for_config(config: &NoMistakesConfig) -> Self {
        let rules = config.rule_applications(super::super::RULE_ID);
        let all = all_runner_projects(config);
        Self {
            vitest: all || rules.iter().any(|rule| !rule.tests.vitest.is_empty()),
            jest: all,
        }
    }
}

pub(super) fn all_runner_projects(config: &NoMistakesConfig) -> bool {
    let rules = config.rule_applications(super::super::RULE_ID);
    rules.is_empty()
        || rules
            .iter()
            .any(|rule| rule_selects_all_runners(config, rule))
}

fn rule_selects_all_runners(config: &NoMistakesConfig, rule: &RuleDef) -> bool {
    // Application scope narrows an explicit runner selection; it does not turn
    // that selection into a broad Vitest/Jest execution.
    rule.tests.vitest.is_empty()
        && rule.tests.playwright.is_empty()
        && (rule.applies_to_repository()
            || rule
                .projects
                .iter()
                .any(|name| config.projects.contains_key(name)))
}

/// Analyze each matching named Vitest project and Jest config separately.
/// Only setup files from the executing runner can cover its dynamic imports.
pub(in super::super) fn setup_contexts_for_test_precomputed(
    rel_path: &str,
    config_data: &[ConfigSetupData],
    selected: SelectedRunners,
) -> Vec<Vec<PathBuf>> {
    let mut vitest_configs = Vec::new();
    let mut jest_contexts = Vec::new();
    let mut projects = Vec::new();
    let mut playwright = false;
    for data in config_data {
        if !data.filter_matches(rel_path) {
            continue;
        }
        if data.explicit_project {
            projects.push(data);
        } else if data.runner == super::discovery::Runner::Vitest {
            vitest_configs.push(data);
        } else if data.runner == super::discovery::Runner::Jest {
            jest_contexts.push(data.setup_files.clone());
        } else if data.runner == super::discovery::Runner::Playwright {
            playwright = true;
        }
    }
    let mut contexts = Vec::new();
    if selected.vitest {
        if projects.is_empty() {
            contexts.extend(vitest_configs.iter().map(|data| data.setup_files.clone()));
        } else {
            contexts.extend(
                vitest_configs
                    .iter()
                    .filter(|data| {
                        (data.project_name.is_some() || selected.includes_all())
                            && !projects
                                .iter()
                                .any(|project| project.project_name == data.project_name)
                    })
                    .map(|data| data.setup_files.clone()),
            );
            for project in projects {
                let configs = vitest_configs
                    .iter()
                    .filter(|data| {
                        data.project_name.is_none() || data.project_name == project.project_name
                    })
                    .collect::<Vec<_>>();
                if configs.is_empty() {
                    contexts.push(project.setup_files.clone());
                }
                for config in configs {
                    let mut files = config.setup_files.clone();
                    files.extend(project.setup_files.iter().cloned());
                    let mut seen = std::collections::HashSet::new();
                    files.retain(|path| seen.insert(path.clone()));
                    contexts.push(files);
                }
            }
        }
    }
    if selected.jest {
        contexts.extend(jest_contexts);
    }
    if playwright {
        contexts.push(Vec::new());
    }
    if contexts.is_empty() {
        contexts.push(Vec::new());
    }
    contexts
}
