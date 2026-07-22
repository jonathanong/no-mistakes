// Full-suite trigger logic for configured test plans.
//
// Legacy triggers request a framework-wide fallback. Structured triggers
// select named runner projects. Pattern lists deliberately use ordered
// last-match-wins semantics so a later positive pattern can re-include a path
// excluded by `!pattern`.

use super::super::plan::relative_path;
use super::TestFramework;
use anyhow::Result;
use globset::{GlobBuilder, GlobMatcher};
use no_mistakes::codebase::test_discovery::TestRunner;
use no_mistakes::config::v2::schema::{
    NoMistakesConfig, Project, TestPlanIgnoredChangedTestsFramework, TestPlanProjectDependency,
};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub(super) struct DependencyTriggers {
    pub(super) fallback: Option<(String, PathBuf)>,
    /// Changed files that selected each runner-project name. A `BTree*` shape
    /// makes both selection and rendered reasons deterministic.
    pub(super) targeted: BTreeMap<PathBuf, BTreeSet<String>>,
}

pub(super) fn dependency_triggers(
    root: &Path,
    config: &NoMistakesConfig,
    framework: TestFramework,
    changed_files: &[PathBuf],
    prepared: &crate::tests::prepared_plan::PreparedTestPlanRequest,
) -> Result<DependencyTriggers> {
    let plan = framework_plan(config, framework);
    validate_targeted_targets(config, framework, prepared)?;
    let ignored_sets = ignored_changed_test_sets(
        root,
        &plan.full_suite_triggers.ignore_changed_tests,
        changed_files,
        prepared,
    )?;
    let mut result = DependencyTriggers::default();
    let mut legacy_match = None;
    for (project_name, trigger) in &plan.full_suite_triggers.projects {
        let Some(project) = config.projects.get(project_name) else {
            continue;
        };
        let patterns = project_dependency_patterns(project_name, project, trigger);
        for changed in changed_files {
            if ignored_sets.iter().any(|set| set.contains(changed)) {
                continue;
            }
            let rel = relative_path(root, changed);
            if !matches_ordered(&patterns, &rel)? {
                continue;
            }
            match trigger {
                TestPlanProjectDependency::Targeted(targeted) => {
                    result
                        .targeted
                        .entry(changed.clone())
                        .or_default()
                        .extend(targeted.targets.iter().cloned());
                }
                // A matching legacy trigger always wins over every targeted
                // trigger, regardless of resource-project map order.
                TestPlanProjectDependency::All(_) | TestPlanProjectDependency::Patterns(_) => {
                    legacy_match.get_or_insert_with(|| {
                        (
                            format!("{} project dependency changed: {}", project_name, rel),
                            changed.clone(),
                        )
                    });
                }
            }
        }
    }
    if legacy_match.is_some() {
        result.targeted.clear();
    }
    result.fallback = legacy_match;
    Ok(result)
}

fn framework_plan(
    config: &NoMistakesConfig,
    framework: TestFramework,
) -> &no_mistakes::config::v2::schema::TestPlanFrameworkConfig {
    match framework {
        TestFramework::Dotnet => &config.test_plan.dotnet,
        TestFramework::Playwright => &config.test_plan.playwright,
        TestFramework::Vitest => &config.test_plan.vitest,
        TestFramework::Swift => &config.test_plan.swift,
    }
}

fn validate_targeted_targets(
    config: &NoMistakesConfig,
    framework: TestFramework,
    prepared: &crate::tests::prepared_plan::PreparedTestPlanRequest,
) -> Result<()> {
    let runner = test_runner(framework);
    let projects = prepared.requested_runner_projects(runner)?;
    for (resource_project, dependency) in &framework_plan(config, framework)
        .full_suite_triggers
        .projects
    {
        let TestPlanProjectDependency::Targeted(targeted) = dependency else {
            continue;
        };
        for (index, target) in targeted.targets.iter().enumerate() {
            let matching = projects
                .iter()
                .filter(|project| project.runner_project_arg.as_deref() == Some(target.as_str()))
                .collect::<Vec<_>>();
            let field = format!(
                "{}.testPlan.{}.fullSuiteTriggers.projects.{resource_project}.targets[{index}]",
                prepared.config_path().map_or_else(
                    || "<in-memory config>".to_string(),
                    |path| path.display().to_string()
                ),
                runner.as_str()
            );
            match matching.len() {
                1 => {}
                0 => anyhow::bail!(
                    "{field} references unknown {} runner project `{target}`",
                    runner.as_str()
                ),
                _ => anyhow::bail!(
                    "{field} references ambiguous {} runner project `{target}` in configs: {}",
                    runner.as_str(),
                    matching
                        .iter()
                        .map(|project| project.config.as_deref().unwrap_or("<default>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            }
        }
    }
    Ok(())
}

fn ignored_changed_test_sets(
    root: &Path,
    ignored: &[TestPlanIgnoredChangedTestsFramework],
    changed_files: &[PathBuf],
    prepared: &crate::tests::prepared_plan::PreparedTestPlanRequest,
) -> Result<Vec<HashSet<PathBuf>>> {
    let mut sets = Vec::new();
    for framework in ignored {
        let runner = match framework {
            TestPlanIgnoredChangedTestsFramework::Dotnet => TestRunner::Dotnet,
            TestPlanIgnoredChangedTestsFramework::Playwright => TestRunner::Playwright,
            TestPlanIgnoredChangedTestsFramework::Vitest => TestRunner::Vitest,
            TestPlanIgnoredChangedTestsFramework::Swift => TestRunner::Swift,
        };
        let set = match prepared.discover_runner_tests(runner) {
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

pub(super) fn project_dependency_patterns(
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
        TestPlanProjectDependency::Targeted(targeted) => {
            let root = project.root.as_deref().unwrap_or(project_name);
            targeted
                .paths
                .iter()
                .map(|pattern| project_relative_pattern(root, pattern))
                .collect()
        }
    }
}

/// Apply every matching entry in order; the final matching entry decides.
pub(super) fn matches_ordered(patterns: &[String], path: &str) -> Result<bool> {
    let mut matched = false;
    for pattern in patterns {
        let (negated, pattern) = pattern
            .strip_prefix('!')
            .map_or((false, pattern.as_str()), |pattern| (true, pattern));
        let glob = GlobBuilder::new(pattern).literal_separator(false).build()?;
        let matcher: GlobMatcher = glob.compile_matcher();
        if matcher.is_match(path) {
            matched = !negated;
        }
    }
    Ok(matched)
}

fn project_root_patterns(project_root: &str) -> Vec<String> {
    let root = normalize_project_glob_part(project_root);
    if root.is_empty() || root == "." {
        vec!["**".to_string()]
    } else {
        vec![format!("{root}/**")]
    }
}

/// Split `!` before root-prefixing. Prefixing first produces invalid globs
/// such as `pkg/!dist/**` and makes negated entries impossible to re-include.
pub(super) fn project_relative_pattern(project_root: &str, raw_pattern: &str) -> String {
    let (negated, pattern) = raw_pattern
        .trim()
        .strip_prefix('!')
        .map_or((false, raw_pattern.trim()), |pattern| (true, pattern));
    let root = normalize_project_glob_part(project_root);
    let pattern = normalize_project_glob_part(pattern);
    let joined = if root.is_empty() || root == "." || pattern.starts_with(&format!("{root}/")) {
        pattern
    } else {
        format!("{root}/{pattern}")
    };
    if negated {
        format!("!{joined}")
    } else {
        joined
    }
}

fn normalize_project_glob_part(raw: &str) -> String {
    let mut part = raw.trim().trim_matches('/').to_string();
    while let Some(rest) = part.strip_prefix("./") {
        part = rest.to_string();
    }
    part
}

fn test_runner(framework: TestFramework) -> TestRunner {
    match framework {
        TestFramework::Dotnet => TestRunner::Dotnet,
        TestFramework::Playwright => TestRunner::Playwright,
        TestFramework::Vitest => TestRunner::Vitest,
        TestFramework::Swift => TestRunner::Swift,
    }
}
