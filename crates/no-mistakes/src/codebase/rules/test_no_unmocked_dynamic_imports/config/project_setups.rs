use super::{build_globset, ConfigSetupData, TestFilter};
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
    let all_projects = rules.is_empty()
        || rules.iter().any(|rule| {
            rule.applies_to_repository()
                || rule
                    .projects
                    .iter()
                    .any(|project| config.projects.contains_key(project))
        });
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
        anyhow::ensure!(
            !project.include.is_empty(),
            "tests.vitest.projects.{name}.setup_files requires a nonempty include matcher"
        );
        let filter = TestFilter {
            include: build_globset(&project.include)?,
            include_regex: Vec::new(),
            exclude: build_globset(&project.exclude)?,
        };
        let mut setup_files = Vec::new();
        for setup in &project.setup_files {
            let relative = Path::new(setup);
            anyhow::ensure!(
                !setup.is_empty()
                    && !relative.is_absolute()
                    && relative.components().all(|part| matches!(part, std::path::Component::Normal(_) | std::path::Component::CurDir)),
                "tests.vitest.projects.{name}.setup_files contains invalid repository-relative path {setup:?}"
            );
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
            filter,
            setup_files,
            explicit_project: true,
            runner: super::discovery::Runner::Vitest,
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
    pub(in super::super) fn for_config(config: &NoMistakesConfig) -> Self {
        let rules = config.rule_applications(super::super::RULE_ID);
        let all = rules.is_empty()
            || rules.iter().any(|rule| {
                rule.applies_to_repository()
                    || rule
                        .projects
                        .iter()
                        .any(|project| config.projects.contains_key(project))
            });
        Self {
            vitest: all || rules.iter().any(|rule| !rule.tests.vitest.is_empty()),
            jest: all,
        }
    }
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
    for data in config_data {
        if !data.filter_matches(rel_path) {
            continue;
        }
        if data.explicit_project {
            projects.push(data.setup_files.clone());
        } else if data.runner == super::discovery::Runner::Vitest {
            vitest_configs.push(data.setup_files.clone());
        } else if data.runner == super::discovery::Runner::Jest {
            jest_contexts.push(data.setup_files.clone());
        }
    }
    let mut contexts = Vec::new();
    if selected.vitest {
        if projects.is_empty() {
            contexts.extend(vitest_configs);
        } else {
            for project in projects {
                if vitest_configs.is_empty() {
                    contexts.push(project);
                    continue;
                }
                for config_files in &vitest_configs {
                    let mut files = config_files.clone();
                    files.extend(project.iter().cloned());
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
    if contexts.is_empty() {
        contexts.push(Vec::new());
    }
    contexts
}
