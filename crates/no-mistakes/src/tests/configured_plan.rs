use super::configured_plan_candidates::{
    group_candidates, merge_selected, missing_file_warnings, selected_from_paths, stable_take,
};
use super::plan::relative_path;
use super::{PlanArgs, SelectedTest, TestFramework, TestPlan, TestPlanGroupResult, Warning};
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use no_mistakes::codebase::dependencies::graph::DepGraph;
use no_mistakes::codebase::test_filter::TestFileFilter;
use no_mistakes::config::v2::schema::{
    NoMistakesConfig, Project, TestPlanEnvironment, TestPlanGroup, TestPlanGroupType,
    TestPlanLimit, TestPlanProjectDependency,
};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

pub(crate) fn generate_configured_plan(
    args: &PlanArgs,
    framework: TestFramework,
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig: &no_mistakes::codebase::dependencies::TsConfig,
    changed_files: &[PathBuf],
    forced_fallback: Option<(String, PathBuf)>,
) -> Result<TestPlan> {
    let env = configured_environment(args, framework, config)?;
    let all_tests = discover_framework_tests(root, config, framework, &env)?;
    let all_test_set: HashSet<PathBuf> = all_tests.iter().cloned().collect();
    let effective_limit = override_limit(env.limit.as_ref(), args);
    let global_limit = limit_count(effective_limit.as_ref(), all_tests.len()).unwrap_or(usize::MAX);

    if let Some((reason, trigger_file)) = forced_fallback.as_ref() {
        return Ok(TestPlan {
            selected_tests: selected_from_paths(
                root,
                &all_tests,
                "global configuration",
                Some(trigger_file),
            ),
            groups: vec![TestPlanGroupResult {
                r#type: "global".to_string(),
                ..all_group(root, &all_tests)
            }],
            warnings: missing_file_warnings(root, changed_files),
            fallback_triggered: true,
            fallback_reason: Some(reason.clone()),
        });
    }

    if env.all {
        return Ok(TestPlan {
            selected_tests: selected_from_paths(root, &all_tests, "all", changed_files.first()),
            groups: vec![all_group(root, &all_tests)],
            warnings: missing_file_warnings(root, changed_files),
            fallback_triggered: true,
            fallback_reason: Some(format!(
                "{} test plan environment `{}` runs all tests",
                framework_name(framework),
                args.environment
            )),
        });
    }

    if let Some((reason, trigger_file)) =
        dependency_trigger(root, config, framework, changed_files)?
    {
        return Ok(TestPlan {
            selected_tests: selected_from_paths(
                root,
                &all_tests,
                "dependency configuration",
                Some(&trigger_file),
            ),
            groups: vec![TestPlanGroupResult {
                r#type: "dependencies".to_string(),
                ..all_group(root, &all_tests)
            }],
            warnings: missing_file_warnings(root, changed_files),
            fallback_triggered: true,
            fallback_reason: Some(reason),
        });
    }

    let graph = DepGraph::build(root, tsconfig)?;
    let mut selected_map: BTreeMap<PathBuf, SelectedTest> = BTreeMap::new();
    let mut used = HashSet::new();
    let mut group_results = Vec::new();
    let mut remaining_global = global_limit;
    let mut warnings = missing_file_warnings(root, changed_files);
    let mut warnings_seen: HashSet<(String, String)> = warnings
        .iter()
        .map(|warning| (warning.r#type.clone(), warning.file.clone()))
        .collect();
    let groups = configured_groups(&env, framework);

    for group in &groups {
        if remaining_global == 0 {
            group_results.push(empty_group_result(
                group.type_,
                all_tests.len().saturating_sub(used.len()),
            ));
            continue;
        }
        if framework == TestFramework::Vitest && group.type_ == TestPlanGroupType::Coverage {
            anyhow::bail!("vitest test plans do not support the coverage group");
        }
        let candidates = group_candidates(
            group.type_,
            root,
            changed_files,
            &graph,
            &all_tests,
            &all_test_set,
            &used,
            &mut warnings,
            &mut warnings_seen,
        );
        let group_limit = group
            .limit
            .as_ref()
            .and_then(|limit| limit_count(Some(limit), all_tests.len()))
            .unwrap_or(remaining_global)
            .min(remaining_global);
        let picked = stable_take(candidates, group_limit);
        for test in &picked {
            used.insert(test.test_file.clone());
            selected_map
                .entry(root.join(&test.test_file))
                .and_modify(|entry| merge_selected(entry, test))
                .or_insert_with(|| test.clone());
        }
        remaining_global = remaining_global.saturating_sub(picked.len());
        group_results.push(TestPlanGroupResult {
            r#type: group_type_name(group.type_).to_string(),
            selected: picked.iter().map(|test| test.test_file.clone()).collect(),
            remaining: all_tests.len().saturating_sub(used.len()),
            limit: Some(group_limit),
        });
    }

    Ok(TestPlan {
        selected_tests: sorted_selected_tests(selected_map),
        groups: group_results,
        warnings: sorted_warnings(warnings),
        fallback_triggered: false,
        fallback_reason: None,
    })
}

fn all_group(root: &Path, all_tests: &[PathBuf]) -> TestPlanGroupResult {
    TestPlanGroupResult {
        r#type: "all".to_string(),
        selected: all_tests
            .iter()
            .map(|test| relative_path(root, test))
            .collect(),
        remaining: 0,
        limit: None,
    }
}

fn empty_group_result(group: TestPlanGroupType, remaining: usize) -> TestPlanGroupResult {
    TestPlanGroupResult {
        r#type: group_type_name(group).to_string(),
        selected: Vec::new(),
        remaining,
        limit: Some(0),
    }
}

fn sorted_selected_tests(selected_map: BTreeMap<PathBuf, SelectedTest>) -> Vec<SelectedTest> {
    let mut selected_tests: Vec<SelectedTest> = selected_map.into_values().collect();
    for test in &mut selected_tests {
        test.reasons
            .sort_by(|a, b| a.changed_file.cmp(&b.changed_file));
    }
    selected_tests.sort_by(|a, b| a.test_file.cmp(&b.test_file));
    selected_tests
}

fn sorted_warnings(mut warnings: Vec<Warning>) -> Vec<Warning> {
    warnings.sort_by(|a, b| (&a.file, &a.message).cmp(&(&b.file, &b.message)));
    warnings
}

fn configured_environment(
    args: &PlanArgs,
    framework: TestFramework,
    config: &NoMistakesConfig,
) -> Result<TestPlanEnvironment> {
    let plan = match framework {
        TestFramework::Playwright => &config.test_plan.playwright,
        TestFramework::Vitest => &config.test_plan.vitest,
    };
    let key = normalize_environment(&args.environment);
    for (name, env) in &plan.environments {
        if normalize_environment(name) == key {
            return Ok(env.clone());
        }
    }
    Ok(TestPlanEnvironment {
        groups: default_groups(framework),
        ..TestPlanEnvironment::default()
    })
}

fn normalize_environment(raw: &str) -> String {
    raw.chars()
        .filter(|ch| *ch != '-' && *ch != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

fn configured_groups(env: &TestPlanEnvironment, framework: TestFramework) -> Vec<TestPlanGroup> {
    if env.groups.is_empty() {
        default_groups(framework)
    } else {
        env.groups.clone()
    }
}

fn default_groups(framework: TestFramework) -> Vec<TestPlanGroup> {
    let mut groups = vec![TestPlanGroup {
        type_: TestPlanGroupType::Direct,
        limit: None,
    }];
    if framework == TestFramework::Playwright {
        groups.push(TestPlanGroup {
            type_: TestPlanGroupType::Coverage,
            limit: None,
        });
    }
    groups.push(TestPlanGroup {
        type_: TestPlanGroupType::Dependencies,
        limit: None,
    });
    groups
}

fn framework_name(framework: TestFramework) -> &'static str {
    match framework {
        TestFramework::Playwright => "playwright",
        TestFramework::Vitest => "vitest",
    }
}

fn group_type_name(group: TestPlanGroupType) -> &'static str {
    match group {
        TestPlanGroupType::Direct => "direct",
        TestPlanGroupType::Coverage => "coverage",
        TestPlanGroupType::Dependencies => "dependencies",
        TestPlanGroupType::Sample => "sample",
    }
}

fn override_limit(limit: Option<&TestPlanLimit>, args: &PlanArgs) -> Option<TestPlanLimit> {
    let mut next = limit.cloned().unwrap_or_default();
    if let Some(percent) = args.limit_percent {
        next.percent = Some(no_mistakes::config::v2::schema::TestPlanPercent::Number(
            percent,
        ));
    }
    if let Some(files) = args.limit_files {
        next.files = Some(files);
    }
    (next.percent.is_some() || next.files.is_some()).then_some(next)
}

fn limit_count(limit: Option<&TestPlanLimit>, total: usize) -> Option<usize> {
    let limit = limit?;
    let percent = limit.percent.as_ref().and_then(|percent| percent.value());
    let percent_files = percent.map(|percent| ((total as f64) * percent / 100.0).ceil() as usize);
    match (percent_files, limit.files) {
        (Some(percent), Some(files)) => Some(percent.min(files)),
        (Some(percent), None) => Some(percent),
        (None, Some(files)) => Some(files),
        (None, None) => None,
    }
}

fn discover_framework_tests(
    root: &Path,
    config: &NoMistakesConfig,
    framework: TestFramework,
    env: &TestPlanEnvironment,
) -> Result<Vec<PathBuf>> {
    if framework == TestFramework::Playwright {
        return discover_playwright_tests(root, config, env);
    }

    let playwright_tests: HashSet<PathBuf> = discover_playwright_tests(root, config, env)?
        .into_iter()
        .collect();
    let include = compile_globset(&env.include)?;
    let exclude = compile_globset(&env.exclude)?;
    let filter = TestFileFilter::new(root, config);
    let mut tests: Vec<PathBuf> =
        no_mistakes::codebase::ts_source::discover_files(root, &config.filesystem.skip_directories)
            .into_iter()
            .filter(|path| {
                let rel = relative_path(root, path);
                framework_test_match(framework, &rel)
                    && filter.is_match(root, path)
                    && !playwright_tests.contains(path)
                    && include.as_ref().is_none_or(|set| set.is_match(&rel))
                    && exclude.as_ref().is_none_or(|set| !set.is_match(&rel))
            })
            .collect();
    tests.sort();
    Ok(tests)
}

fn discover_playwright_tests(
    root: &Path,
    config: &NoMistakesConfig,
    env: &TestPlanEnvironment,
) -> Result<Vec<PathBuf>> {
    let include = compile_globset(&env.include)?;
    let exclude = compile_globset(&env.exclude)?;
    let mut tests = fallback_playwright_tests(root, config);
    tests.retain(|path| {
        let rel = relative_path(root, path);
        include.as_ref().is_none_or(|set| set.is_match(&rel))
            && exclude.as_ref().is_none_or(|set| !set.is_match(&rel))
    });
    tests.sort();
    tests.dedup();
    Ok(tests)
}

fn fallback_playwright_tests(root: &Path, config: &NoMistakesConfig) -> Vec<PathBuf> {
    let filter = TestFileFilter::new(root, config);
    no_mistakes::codebase::ts_source::discover_files(root, &config.filesystem.skip_directories)
        .into_iter()
        .filter(|path| {
            let rel = relative_path(root, path);
            filter.is_match(root, path)
                && (rel.contains("/tests/e2e/")
                    || rel.starts_with("tests/e2e/")
                    || rel.contains("/playwright/")
                    || rel.starts_with("playwright/")
                    || rel.starts_with("specs/"))
        })
        .collect()
}

fn framework_test_match(framework: TestFramework, rel: &str) -> bool {
    match framework {
        TestFramework::Playwright => unreachable!("playwright tests are discovered separately"),
        TestFramework::Vitest => {
            let name = rel.rsplit('/').next().unwrap_or(rel);
            (rel.split('/').any(|component| component == "__tests__")
                || name.contains(".test.")
                || name.contains(".spec."))
                && !rel.split('/').any(|component| component == "playwright")
                && !has_path_segment_pair(rel, "tests", "e2e")
                && !rel.starts_with("specs/")
        }
    }
}

fn has_path_segment_pair(path: &str, first: &str, second: &str) -> bool {
    let segments = path.split('/').collect::<Vec<_>>();
    segments
        .windows(2)
        .any(|pair| pair[0] == first && pair[1] == second)
}

fn compile_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(Some(builder.build()?))
}

fn dependency_trigger(
    root: &Path,
    config: &NoMistakesConfig,
    framework: TestFramework,
    changed_files: &[PathBuf],
) -> Result<Option<(String, PathBuf)>> {
    let plan = match framework {
        TestFramework::Playwright => &config.test_plan.playwright,
        TestFramework::Vitest => &config.test_plan.vitest,
    };
    for (project_name, trigger) in &plan.dependencies.projects {
        let Some(project) = config.projects.get(project_name) else {
            continue;
        };
        let patterns = project_dependency_patterns(project_name, project, trigger);
        let globset = compile_globset(&patterns)?;
        for changed in changed_files {
            let rel = relative_path(root, changed);
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

fn project_dependency_patterns(
    project_name: &str,
    project: &Project,
    trigger: &TestPlanProjectDependency,
) -> Vec<String> {
    match trigger {
        TestPlanProjectDependency::All(false) => Vec::new(),
        TestPlanProjectDependency::All(true) => {
            let root = project.root.as_deref().unwrap_or(project_name);
            let mut patterns = project_root_patterns(root);
            if project.include.is_empty() {
                patterns
            } else {
                patterns.extend(
                    project
                        .include
                        .iter()
                        .map(|pattern| project_relative_pattern(root, pattern)),
                );
                patterns
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
