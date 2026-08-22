use super::{ResolvedFrameworkTriggers, ResolvedTrigger};
use crate::config::v2::schema::{
    NamedFullSuiteTrigger, NoMistakesConfig, TestPlanFrameworkConfig, TestPlanProjectDependency,
};

pub(super) fn resolved_vitest_triggers(config: &NoMistakesConfig) -> Vec<ResolvedTrigger> {
    resolved_triggers_for(config, &config.test_plan.vitest)
}

pub(super) fn resolved_framework_triggers(
    config: &NoMistakesConfig,
) -> Vec<ResolvedFrameworkTriggers> {
    framework_plans(config)
        .into_iter()
        .filter_map(|(framework, plan)| {
            let triggers = resolved_triggers_for(config, plan);
            (!triggers.is_empty()).then_some(ResolvedFrameworkTriggers {
                framework,
                triggers,
            })
        })
        .collect()
}

fn framework_plans(config: &NoMistakesConfig) -> [(&'static str, &TestPlanFrameworkConfig); 10] {
    [
        ("dotnet", &config.test_plan.dotnet),
        ("playwright", &config.test_plan.playwright),
        ("vitest", &config.test_plan.vitest),
        ("swift", &config.test_plan.swift),
        ("python", &config.test_plan.python),
        ("go", &config.test_plan.go),
        ("cargo", &config.test_plan.cargo),
        ("rails", &config.test_plan.rails),
        ("php", &config.test_plan.php),
        ("jest", &config.test_plan.jest),
    ]
}

fn resolved_triggers_for(
    config: &NoMistakesConfig,
    plan: &TestPlanFrameworkConfig,
) -> Vec<ResolvedTrigger> {
    let mut triggers = plan
        .full_suite_triggers
        .triggers
        .iter()
        .map(named_trigger)
        .collect::<Vec<_>>();
    for (name, dependency) in &plan.full_suite_triggers.projects {
        if let Some(trigger) = project_trigger(config, name, dependency) {
            triggers.push(trigger);
        }
    }
    triggers
}

fn named_trigger(trigger: &NamedFullSuiteTrigger) -> ResolvedTrigger {
    ResolvedTrigger {
        name: trigger.name.clone(),
        paths: trigger
            .paths
            .iter()
            .map(|pattern| project_relative_pattern(".", pattern))
            .collect(),
        targets: trigger.targets.clone(),
        source: "triggers",
    }
}

fn project_trigger(
    config: &NoMistakesConfig,
    name: &str,
    dependency: &TestPlanProjectDependency,
) -> Option<ResolvedTrigger> {
    if !config.projects.contains_key(name) {
        return None;
    }
    Some(match dependency {
        TestPlanProjectDependency::All(false) => return None,
        TestPlanProjectDependency::All(true) => ResolvedTrigger {
            name: name.to_string(),
            paths: Vec::new(),
            targets: Vec::new(),
            source: "projects",
        },
        TestPlanProjectDependency::Patterns(paths) => ResolvedTrigger {
            name: name.to_string(),
            paths: expand_project_paths(config, name, paths),
            targets: Vec::new(),
            source: "projects",
        },
        TestPlanProjectDependency::Targeted(targeted) => ResolvedTrigger {
            name: name.to_string(),
            paths: expand_project_paths(config, name, &targeted.paths),
            targets: targeted.targets.clone(),
            source: "projects",
        },
    })
}

fn expand_project_paths(
    config: &NoMistakesConfig,
    project_name: &str,
    paths: &[String],
) -> Vec<String> {
    let root = config
        .projects
        .get(project_name)
        .and_then(|project| project.root.as_deref())
        .unwrap_or(project_name);
    paths
        .iter()
        .map(|pattern| project_relative_pattern(root, pattern))
        .collect()
}

fn project_relative_pattern(project_root: &str, raw_pattern: &str) -> String {
    let trimmed = raw_pattern.trim();
    let (negated, pattern) = trimmed
        .strip_prefix('!')
        .map_or((false, trimmed), |pattern| (true, pattern.trim()));
    let root = normalize_glob_part(project_root);
    let pattern = normalize_glob_part(pattern);
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

fn normalize_glob_part(raw: &str) -> String {
    let mut part = raw.trim().trim_matches('/').to_string();
    while let Some(rest) = part.strip_prefix("./") {
        part = rest.to_string();
    }
    part
}
