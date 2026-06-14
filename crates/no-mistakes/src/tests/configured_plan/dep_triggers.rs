// Full-suite trigger logic for configured test plans.
// Determines whether a changed file matches a project-level dependency pattern
// that should force a full-suite run.

use super::super::plan::relative_path;
use super::{compile_globset, TestFramework};
use anyhow::Result;
use no_mistakes::codebase::test_discovery::TestRunner;
use no_mistakes::config::v2::schema::{
    NoMistakesConfig, Project, TestPlanIgnoredChangedTestsFramework, TestPlanProjectDependency,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn dependency_trigger(
    root: &Path,
    config: &NoMistakesConfig,
    framework: TestFramework,
    changed_files: &[PathBuf],
) -> Result<Option<(String, PathBuf)>> {
    let plan = match framework {
        TestFramework::Playwright => &config.test_plan.playwright,
        TestFramework::Vitest => &config.test_plan.vitest,
        TestFramework::Swift => &config.test_plan.swift,
    };
    let ignored_sets = ignored_changed_test_sets(
        root,
        config,
        &plan.full_suite_triggers.ignore_changed_tests,
        changed_files,
    )?;
    for (project_name, trigger) in &plan.full_suite_triggers.projects {
        let Some(project) = config.projects.get(project_name) else {
            continue;
        };
        let patterns = project_dependency_patterns(project_name, project, trigger);
        let globset = compile_globset(&patterns)?;
        for changed in changed_files {
            let rel = relative_path(root, changed);
            if ignored_sets.iter().any(|set| set.contains(changed)) {
                continue;
            }
            if globset.as_ref().is_some_and(|set| set.is_match(&rel)) {
                return Ok(Some((
                    format!("{} project dependency changed: {}", project_name, rel),
                    changed.clone(),
                )));
            }
        }
    }
    Ok(None)
}

fn ignored_changed_test_sets(
    root: &Path,
    config: &NoMistakesConfig,
    ignored: &[TestPlanIgnoredChangedTestsFramework],
    changed_files: &[PathBuf],
) -> Result<Vec<HashSet<PathBuf>>> {
    let mut sets = Vec::new();
    for framework in ignored {
        let runner = match framework {
            TestPlanIgnoredChangedTestsFramework::Playwright => TestRunner::Playwright,
            TestPlanIgnoredChangedTestsFramework::Vitest => TestRunner::Vitest,
            TestPlanIgnoredChangedTestsFramework::Swift => TestRunner::Swift,
        };
        let set = match no_mistakes::codebase::test_discovery::discover_tests(root, config, runner)
        {
            Ok(discovered) => discovered.tests.into_iter().collect(),
            Err(_) => changed_files
                .iter()
                .filter(|path| {
                    let rel = relative_path(root, path);
                    no_mistakes::codebase::test_discovery::fallback_runner_match(runner, &rel)
                })
                .cloned()
                .collect(),
        };
        sets.push(set);
    }
    Ok(sets)
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
