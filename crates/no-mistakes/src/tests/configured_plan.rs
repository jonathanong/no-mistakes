use super::configured_plan_candidates::{
    first_take, group_candidates, merge_selected, stable_take,
};
use super::diff_parser::DiffFile;
use super::plan::relative_path;
use super::{PlanArgs, SelectedTest, TestFramework, TestPlan, TestPlanGroupResult, Warning};
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use no_mistakes::codebase::dependencies::graph::DepGraph;
use no_mistakes::codebase::test_discovery::{DiscoveredTests, TestRunner};
use no_mistakes::codebase::test_filter::TestFileFilter;
use no_mistakes::config::v2::schema::{
    NoMistakesConfig, Project, TestPlanEnvironment, TestPlanGroup, TestPlanGroupType,
    TestPlanIgnoredChangedTestsFramework, TestPlanLimit, TestPlanProjectDependency,
};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

mod fallback;
mod hints;
mod hints_domains;
#[cfg(test)]
mod tests;
use fallback::{fallback_plan, FallbackRequest};
use hints::build_coverage_hints;

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_configured_plan(
    args: &PlanArgs,
    framework: TestFramework,
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig: &no_mistakes::codebase::dependencies::TsConfig,
    changed_files: &[PathBuf],
    diff_files: &[DiffFile],
    forced_fallback: Option<(String, PathBuf)>,
) -> Result<TestPlan> {
    let env = configured_environment(args, framework, config)?;
    let discovered_tests = discover_framework_tests(root, config, framework, &env)?;
    let all_tests = discovered_tests.tests.clone();
    let all_test_set: HashSet<PathBuf> = all_tests.iter().cloned().collect();
    let effective_limit = override_limit(env.limit.as_ref(), args);
    let has_global_limit = effective_limit.is_some();
    let global_limit =
        limit_count(effective_limit.as_ref(), all_tests.len()).unwrap_or(all_tests.len());

    if effective_global_config_fallback(&env, args) {
        if let Some((reason, trigger_file)) = forced_fallback.as_ref() {
            let mut plan = fallback_plan(
                root,
                &all_tests,
                FallbackRequest {
                    group_type: "global",
                    via: "global configuration",
                    changed_file: Some(trigger_file),
                    limit: global_limit,
                    has_limit: has_global_limit,
                    reason: reason.clone(),
                },
            );
            attach_targets(&mut plan, root, &discovered_tests);
            return Ok(plan);
        }
    }

    if env.all {
        let mut plan = fallback_plan(
            root,
            &all_tests,
            FallbackRequest {
                group_type: "all",
                via: "all",
                changed_file: changed_files.first(),
                limit: global_limit,
                has_limit: has_global_limit,
                reason: format!(
                    "{} test plan environment `{}` runs all tests",
                    framework_name(framework),
                    args.environment
                ),
            },
        );
        attach_targets(&mut plan, root, &discovered_tests);
        return Ok(plan);
    }

    if let Some((reason, trigger_file)) =
        dependency_trigger(root, config, framework, changed_files)?
    {
        let mut plan = fallback_plan(
            root,
            &all_tests,
            FallbackRequest {
                group_type: "dependencies",
                via: "dependency configuration",
                changed_file: Some(&trigger_file),
                limit: global_limit,
                has_limit: has_global_limit,
                reason,
            },
        );
        attach_targets(&mut plan, root, &discovered_tests);
        return Ok(plan);
    }

    let graph = DepGraph::build(root, tsconfig)?;
    let test_filter = TestFileFilter::new(root, config);
    let coverage_hints = build_coverage_hints(
        root,
        args.config.as_deref(),
        config,
        framework,
        diff_files,
        &all_tests,
    );
    let mut selected_map: BTreeMap<PathBuf, SelectedTest> = BTreeMap::new();
    let mut used = HashSet::new();
    let mut group_results = Vec::new();
    let mut remaining_global = global_limit;
    let mut warnings: Vec<Warning> = Vec::new();
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
                has_global_limit.then_some(0),
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
            &test_filter,
            &used,
            &coverage_hints,
            &mut warnings,
            &mut warnings_seen,
        );
        let group_limit = group
            .limit
            .as_ref()
            .and_then(|limit| limit_count(Some(limit), all_tests.len()))
            .unwrap_or(remaining_global)
            .min(remaining_global);
        let picked =
            select_limited_group_candidates(candidates, group_limit, group.sample_when_limited);
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
            limit: group
                .limit
                .is_some()
                .then_some(group_limit)
                .or_else(|| has_global_limit.then_some(group_limit)),
        });
    }

    let mut plan = TestPlan {
        selected_tests: sorted_selected_tests(selected_map),
        groups: group_results,
        warnings: sorted_warnings(warnings),
        fallback_triggered: false,
        fallback_reason: None,
    };
    attach_targets(&mut plan, root, &discovered_tests);
    Ok(plan)
}

fn empty_group_result(
    group: TestPlanGroupType,
    remaining: usize,
    limit: Option<usize>,
) -> TestPlanGroupResult {
    TestPlanGroupResult {
        r#type: group_type_name(group).to_string(),
        selected: Vec::new(),
        remaining,
        limit,
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

fn select_limited_group_candidates(
    candidates: Vec<SelectedTest>,
    limit: usize,
    sample_when_limited: bool,
) -> Vec<SelectedTest> {
    if limit == 0 {
        Vec::new()
    } else if sample_when_limited && candidates.len() > limit {
        stable_take(candidates, limit)
    } else {
        first_take(candidates, limit)
    }
}

fn effective_global_config_fallback(env: &TestPlanEnvironment, args: &PlanArgs) -> bool {
    args.global_config_fallback
        .or(env.global_config_fallback)
        .unwrap_or(false)
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
        sample_when_limited: false,
    }];
    if framework == TestFramework::Playwright {
        groups.push(TestPlanGroup {
            type_: TestPlanGroupType::Coverage,
            limit: None,
            sample_when_limited: false,
        });
    }
    groups.push(TestPlanGroup {
        type_: TestPlanGroupType::Dependencies,
        limit: None,
        sample_when_limited: false,
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
) -> Result<DiscoveredTests> {
    let runner = test_runner(framework);
    let mut discovered =
        no_mistakes::codebase::test_discovery::discover_tests(root, config, runner)?;
    let include = compile_globset(&env.include)?;
    let exclude = compile_globset(&env.exclude)?;
    discovered.tests.retain(|path| {
        let rel = relative_path(root, path);
        include.as_ref().is_none_or(|set| set.is_match(&rel))
            && exclude.as_ref().is_none_or(|set| !set.is_match(&rel))
    });
    let allowed: HashSet<PathBuf> = discovered.tests.iter().cloned().collect();
    discovered
        .targets_by_path
        .retain(|path, _| allowed.contains(path));
    Ok(discovered)
}

fn test_runner(framework: TestFramework) -> TestRunner {
    match framework {
        TestFramework::Playwright => TestRunner::Playwright,
        TestFramework::Vitest => TestRunner::Vitest,
    }
}

fn attach_targets(plan: &mut TestPlan, root: &Path, discovered: &DiscoveredTests) {
    for test in &mut plan.selected_tests {
        let path = root.join(&test.test_file);
        if let Some(targets) = discovered.targets_by_path.get(&path) {
            test.targets = targets.clone();
        }
    }
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
