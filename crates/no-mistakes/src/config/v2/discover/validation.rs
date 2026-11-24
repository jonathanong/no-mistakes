use super::{check_commands, path_globs, swift_commands, targeted_triggers, NoMistakesConfig};
use anyhow::Result;
use std::path::Path;

pub(super) fn validate_v2_config(config: &NoMistakesConfig, path: &Path) -> Result<()> {
    use crate::config::source_locations::ConfigPath;
    for (name, project) in &config.projects {
        for (key, patterns) in [("include", &project.include), ("exclude", &project.exclude)] {
            path_globs::validate_globs(patterns, &format!("projects.{name}.{key}"), |index| {
                ConfigPath::default()
                    .key("projects")
                    .key(name)
                    .key(key)
                    .index(index)
            })?;
        }
    }
    for (index, rule) in config.rules.iter().enumerate() {
        if rule.rule.trim().is_empty() {
            anyhow::bail!("rules[{index}].rule is required");
        }
        for (key, patterns) in [("include", &rule.include), ("exclude", &rule.exclude)] {
            path_globs::validate_globs(
                patterns,
                &format!("rules[{index}] ({}).{key}", rule.rule),
                |pattern| {
                    ConfigPath::default()
                        .key("rules")
                        .index(index)
                        .key(key)
                        .index(pattern)
                },
            )?;
        }
    }
    validate_queue_policies(&config.queues)?;
    check_commands::validate(config)?;
    swift_commands::validate(&config.tests.swift)?;
    for (name, project) in &config.tests.vitest.projects {
        project.validate_vitest_setup_files(name)?;
    }
    targeted_triggers::validate(config, path)?;
    path_globs::validate_playwright_selector_wrappers(&config.tests.playwright.selectors.wrappers)?;
    Ok(())
}

fn validate_queue_policies(queues: &crate::config::v2::schema::QueuesTopLevelConfig) -> Result<()> {
    use crate::config::source_locations::ConfigPath;
    if let Some(retention) = &queues.unbounded_job_retention {
        for key in &retention.keys {
            if !matches!(key.as_str(), "removeOnComplete" | "removeOnFail") {
                anyhow::bail!("queues.unboundedJobRetention.keys contains unsupported key `{key}`");
            }
        }
    }
    let policies = [
        (
            "unboundedJobRetention",
            queues
                .unbounded_job_retention
                .as_ref()
                .map(|policy| &policy.allow_paths),
        ),
        (
            "jobStateAsRecord",
            queues
                .job_state_as_record
                .as_ref()
                .map(|policy| &policy.allow_paths),
        ),
    ];
    for (policy, paths) in policies {
        if let Some(paths) = paths {
            path_globs::validate_globs(paths, &format!("queues.{policy}.allowPaths"), |index| {
                ConfigPath::default()
                    .key("queues")
                    .key(policy)
                    .key("allowPaths")
                    .index(index)
            })?;
        }
    }
    Ok(())
}
