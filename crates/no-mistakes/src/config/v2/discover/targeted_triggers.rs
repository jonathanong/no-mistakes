use crate::config::v2::schema::{NoMistakesConfig, TestPlanProjectDependency};
use anyhow::Result;
use globset::GlobBuilder;
use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn validate(config: &NoMistakesConfig, path: &Path) -> Result<()> {
    for (framework, plan) in [
        ("dotnet", &config.test_plan.dotnet),
        ("playwright", &config.test_plan.playwright),
        ("vitest", &config.test_plan.vitest),
        ("swift", &config.test_plan.swift),
    ] {
        for (project, dependency) in &plan.full_suite_triggers.projects {
            let TestPlanProjectDependency::Targeted(targeted) = dependency else {
                continue;
            };
            let base = format!(
                "{}.testPlan.{framework}.fullSuiteTriggers.projects.{project}",
                path.display()
            );
            if !config.projects.contains_key(project) {
                anyhow::bail!("{base} references missing top-level projects.{project}");
            }
            if targeted.paths.is_empty() {
                anyhow::bail!("{base}.paths must not be empty");
            }
            if targeted.targets.is_empty() {
                anyhow::bail!("{base}.targets must not be empty");
            }
            for (index, pattern) in targeted.paths.iter().enumerate() {
                let normalized = pattern.trim();
                let normalized = normalized.strip_prefix('!').unwrap_or(normalized).trim();
                if normalized.is_empty() {
                    anyhow::bail!("{base}.paths[{index}] must not be blank");
                }
                GlobBuilder::new(normalized.trim_start_matches("./"))
                    .literal_separator(false)
                    .build()
                    .map_err(|err| {
                        anyhow::anyhow!(
                            "{base}.paths[{index}] contains invalid glob `{pattern}`: {err}"
                        )
                    })?;
            }
            validate_targets(&targeted.targets, &base)?;
        }
    }
    Ok(())
}

fn validate_targets(targets: &[String], base: &str) -> Result<()> {
    let mut indexes = BTreeMap::new();
    for (index, target) in targets.iter().enumerate() {
        if target.trim().is_empty() {
            anyhow::bail!("{base}.targets[{index}] must not be blank");
        }
        // Runner-project lookup is exact, so duplicate detection uses the
        // same exact (untrimmed) identity.
        if let Some(previous_index) = indexes.insert(target.as_str(), index) {
            anyhow::bail!(
                "{base}.targets[{index}] duplicates targets[{previous_index}] `{target}`"
            );
        }
    }
    Ok(())
}
