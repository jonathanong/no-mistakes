use super::ResolvedTrigger;
use crate::config::v2::schema::{
    NamedFullSuiteTrigger, NoMistakesConfig, TestPlanProjectDependency,
};

pub(super) fn resolved_triggers(config: &NoMistakesConfig) -> Vec<ResolvedTrigger> {
    let mut triggers = config
        .test_plan
        .vitest
        .full_suite_triggers
        .triggers
        .iter()
        .map(named_trigger)
        .collect::<Vec<_>>();
    for (name, dependency) in &config.test_plan.vitest.full_suite_triggers.projects {
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
