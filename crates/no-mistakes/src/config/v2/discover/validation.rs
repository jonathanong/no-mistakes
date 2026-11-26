use super::{check_commands, path_globs, swift_commands, targeted_triggers, NoMistakesConfig};
use crate::config::source_locations::ConfigPath;
use crate::config::v2::analysis_policy::valid_project_sql_policy;
use anyhow::Result;
use std::path::Path;

pub(super) fn validate_v2_config(config: &NoMistakesConfig, path: &Path) -> Result<()> {
    validate_projects(config)?;
    for (index, rule) in config.rules.iter().enumerate() {
        if rule.rule.trim().is_empty() {
            anyhow::bail!("rules[{index}].rule is required");
        }
        for (project_index, project) in rule.projects.iter().enumerate() {
            if !config.projects.contains_key(project) {
                return Err(path_globs::field_error(
                    ConfigPath::default()
                        .key("rules")
                        .index(index)
                        .key("projects")
                        .index(project_index),
                    format!(
                        "rules[{index}].projects[{project_index}]: undeclared project `{project}`"
                    ),
                ));
            }
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
    check_commands::validate(config)?;
    swift_commands::validate(&config.tests.swift)?;
    for (name, project) in &config.tests.vitest.projects {
        project.validate_vitest_setup_files(name)?;
    }
    targeted_triggers::validate(config, path)?;
    path_globs::validate_playwright_selector_wrappers(&config.tests.playwright.selectors.wrappers)?;
    Ok(())
}

fn validate_projects(config: &NoMistakesConfig) -> Result<()> {
    for (name, project) in &config.projects {
        let path = ConfigPath::default().key("projects").key(name);
        for (key, patterns) in [
            ("include", &project.include),
            ("exclude", &project.exclude),
            ("routes", &project.routes),
        ] {
            path_globs::validate_globs(patterns, &format!("projects.{name}.{key}"), |index| {
                path.clone().key(key).index(index)
            })?;
        }
        path_globs::validate_globs(
            &project.queues.wait_scopes,
            &format!("projects.{name}.queues.waitScopes"),
            |index| path.clone().key("queues").key("waitScopes").index(index),
        )?;
        path_globs::validate_globs(
            &project.trpc.routers,
            &format!("projects.{name}.trpc.routers"),
            |index| path.clone().key("trpc").key("routers").index(index),
        )?;
        if let Some(policy) = project
            .postgres
            .as_ref()
            .and_then(|postgres| postgres.unanalyzable_sql.as_deref())
            .filter(|policy| !valid_project_sql_policy(policy))
        {
            return Err(path_globs::field_error(
                path.key("postgres").key("unanalyzableSql"),
                format!("projects.{name}.postgres.unanalyzableSql: unknown policy `{policy}`; use report, advise, ignore, or legacy fail"),
            ));
        }
    }
    Ok(())
}
