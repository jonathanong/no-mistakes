use super::{build_globset, ConfigSetupData, TestFilter};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(in super::super) fn explicit_project_setup_data(
    root: &Path,
    config: &NoMistakesConfig,
    visible_files: Option<&crate::fx::PathSet>,
) -> Result<Vec<ConfigSetupData>> {
    let mut data = Vec::new();
    let rules = config.rule_applications(super::super::RULE_ID);
    let all_projects = rules.is_empty() || rules.iter().any(|rule| rule.applies_to_repository());
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
                visible_files.map_or_else(|| path.is_file(), |visible| visible.contains(&path)),
                "tests.vitest.projects.{name}.setup_files path {setup:?} is missing from the analysis file inventory"
            );
            setup_files.push(path);
        }
        setup_files.sort();
        setup_files.dedup();
        data.push(ConfigSetupData {
            filter,
            setup_files,
            explicit_project: true,
        });
    }
    Ok(data)
}

/// Analyze each matching named project separately while retaining shared
/// runner-config setup files in every execution context.
pub(in super::super) fn setup_contexts_for_test_precomputed(
    rel_path: &str,
    config_data: &[ConfigSetupData],
) -> Vec<Vec<PathBuf>> {
    let mut shared = Vec::new();
    let mut projects = Vec::new();
    for data in config_data {
        if !data.filter_matches(rel_path) {
            continue;
        }
        if data.explicit_project {
            projects.push(data.setup_files.clone());
        } else {
            shared.extend(data.setup_files.iter().cloned());
        }
    }
    shared.sort();
    shared.dedup();
    if projects.is_empty() {
        return vec![shared];
    }
    projects
        .into_iter()
        .map(|project| {
            let mut files = shared.clone();
            files.extend(project);
            files.sort();
            files.dedup();
            files
        })
        .collect()
}
