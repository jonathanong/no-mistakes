mod merge;

use super::Options;
use crate::config::v2::schema::{NoMistakesConfig, Project, TestPlanProjectDependency};
use crate::integration_tests::{
    config as integration_config, project_config, types::ConfigProject, types::Framework,
};
use anyhow::Result;
use merge::merge_explicit_project;
use std::path::Path;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(super) struct CoverageUnit {
    pub(super) project: String,
    pub(super) source: CoverageSource,
    pub(super) patterns: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CoverageSource {
    TestInclude,
    FullSuiteTrigger,
    ConfiguredSource,
}

impl CoverageSource {
    pub(super) fn label(self) -> &'static str {
        match self {
            CoverageSource::TestInclude => "test include",
            CoverageSource::FullSuiteTrigger => "full-suite trigger",
            CoverageSource::ConfiguredSource => "configured source",
        }
    }

    pub(super) fn uses_all_files(self) -> bool {
        self != CoverageSource::TestInclude
    }
}

pub(super) fn coverage_units(
    root: &Path,
    config: &NoMistakesConfig,
    opts: &Options,
) -> Result<Vec<CoverageUnit>> {
    let mut units = Vec::new();
    if opts.include_vitest_project_globs.unwrap_or(true) {
        for project in vitest_projects(root, config, opts)? {
            units.push(CoverageUnit {
                project: project_name(&project),
                source: CoverageSource::TestInclude,
                patterns: include_without_excludes(&project),
            });
        }
    }
    if opts.include_full_suite_triggers.unwrap_or(true) {
        for (project_name, trigger) in &config.test_plan.vitest.full_suite_triggers.projects {
            let Some(project) = config.projects.get(project_name) else {
                continue;
            };
            units.push(CoverageUnit {
                project: project_name.clone(),
                source: CoverageSource::FullSuiteTrigger,
                patterns: project_dependency_patterns(project_name, project, trigger),
            });
        }
    }
    for (project, patterns) in &opts.source_globs_by_project {
        units.push(CoverageUnit {
            project: project.clone(),
            source: CoverageSource::ConfiguredSource,
            patterns: patterns.clone(),
        });
    }
    Ok(units)
}

fn vitest_projects(
    root: &Path,
    config: &NoMistakesConfig,
    opts: &Options,
) -> Result<Vec<ConfigProject>> {
    if opts.explicit_projects_only {
        let projects = explicit_vitest_projects(root, config);
        if projects.is_empty() {
            anyhow::bail!(
                "{} explicitProjectsOnly requires at least one tests.vitest.projects entry with include globs",
                super::RULE_ID
            );
        }
        return Ok(projects);
    }

    let mut projects = if needs_config_projects(root, config) {
        project_config::load_projects(
            root,
            Framework::Vitest,
            config.tests.vitest.configs.as_ref(),
        )?
    } else {
        Vec::new()
    };
    for project in explicit_vitest_projects(root, config) {
        merge_explicit_project(&mut projects, project);
    }
    Ok(projects)
}

fn explicit_vitest_projects(root: &Path, config: &NoMistakesConfig) -> Vec<ConfigProject> {
    config
        .tests
        .vitest
        .projects
        .iter()
        .filter_map(|(project_name, policy)| {
            integration_config::configured_project(root, project_name, policy)
        })
        .collect()
}

fn needs_config_projects(root: &Path, config: &NoMistakesConfig) -> bool {
    config.tests.vitest.configs.is_none()
        || config.tests.vitest.projects.is_empty()
        || config
            .tests
            .vitest
            .configs
            .as_ref()
            .is_some_and(|configs| configs.values().iter().any(|raw| root.join(raw).exists()))
        || config
            .tests
            .vitest
            .projects
            .values()
            .any(|policy| policy.include.is_empty())
}

fn project_name(project: &ConfigProject) -> String {
    project
        .policy_name
        .clone()
        .unwrap_or_else(|| "default".to_string())
}

fn include_without_excludes(project: &ConfigProject) -> Vec<String> {
    let mut patterns = project
        .include
        .iter()
        .map(|pattern| normalize_project_glob_part(pattern))
        .collect::<Vec<_>>();
    patterns.extend(
        project
            .exclude
            .iter()
            .map(|pattern| format!("!{}", normalize_project_glob_part(pattern))),
    );
    patterns
}

fn project_dependency_patterns(
    project_name: &str,
    project: &Project,
    trigger: &TestPlanProjectDependency,
) -> Vec<String> {
    match trigger {
        TestPlanProjectDependency::All(false) => Vec::new(),
        TestPlanProjectDependency::All(true) => {
            let root = project.root.as_deref().unwrap_or(project_name);
            if project.include.is_empty() {
                project_root_patterns(root)
            } else {
                project
                    .include
                    .iter()
                    .map(|pattern| project_relative_pattern(root, pattern))
                    .collect()
            }
        }
        TestPlanProjectDependency::Patterns(patterns) => {
            let root = project.root.as_deref().unwrap_or(project_name);
            patterns
                .iter()
                .map(|pattern| project_relative_pattern(root, pattern))
                .collect()
        }
    }
}

fn project_root_patterns(project_root: &str) -> Vec<String> {
    let root = normalize_project_glob_part(project_root);
    if root.is_empty() || root == "." {
        vec!["**".to_string()]
    } else {
        vec![format!("{root}/**")]
    }
}

fn project_relative_pattern(project_root: &str, pattern: &str) -> String {
    let root = normalize_project_glob_part(project_root);
    let pattern = normalize_project_glob_part(pattern);
    if root.is_empty() || root == "." || pattern.starts_with(&format!("{root}/")) {
        pattern
    } else {
        format!("{root}/{pattern}")
    }
}

fn normalize_project_glob_part(raw: &str) -> String {
    let mut part = raw.trim().trim_matches('/').to_string();
    while let Some(rest) = part.strip_prefix("./") {
        part = rest.to_string();
    }
    part
}
