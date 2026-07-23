use crate::tests::{PlanArgs, TestFramework};
use anyhow::Result;
use no_mistakes::config::v2::schema::{
    NoMistakesConfig, TestPlanEnvironment, TestPlanGroup, TestPlanGroupType, TestPlanLimit,
};

pub(super) fn effective_global_config_fallback(env: &TestPlanEnvironment, args: &PlanArgs) -> bool {
    args.global_config_fallback
        .or(env.global_config_fallback)
        .unwrap_or(false)
}

pub(super) fn configured_environment(
    args: &PlanArgs,
    framework: TestFramework,
    config: &NoMistakesConfig,
) -> Result<TestPlanEnvironment> {
    let plan = match framework {
        TestFramework::Dotnet => &config.test_plan.dotnet,
        TestFramework::Playwright => &config.test_plan.playwright,
        TestFramework::Vitest => &config.test_plan.vitest,
        TestFramework::Swift => &config.test_plan.swift,
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

pub(super) fn configured_groups(
    env: &TestPlanEnvironment,
    framework: TestFramework,
) -> Vec<TestPlanGroup> {
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

pub(super) fn framework_name(framework: TestFramework) -> &'static str {
    match framework {
        TestFramework::Playwright => "playwright",
        TestFramework::Vitest => "vitest",
        TestFramework::Dotnet => "dotnet",
        TestFramework::Swift => "swift",
    }
}

pub(super) fn group_type_name(group: TestPlanGroupType) -> &'static str {
    match group {
        TestPlanGroupType::Direct => "direct",
        TestPlanGroupType::Coverage => "coverage",
        TestPlanGroupType::Dependencies => "dependencies",
        TestPlanGroupType::Sample => "sample",
    }
}

pub(super) fn override_limit(
    limit: Option<&TestPlanLimit>,
    args: &PlanArgs,
) -> Option<TestPlanLimit> {
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

pub(super) fn limit_count(limit: Option<&TestPlanLimit>, total: usize) -> Option<usize> {
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
