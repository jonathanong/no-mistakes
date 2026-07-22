use anyhow::Result;
use globset::GlobBuilder;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::schema::{NoMistakesConfig, TestPlanProjectDependency};
use crate::config::{
    find_automatic_config_path, find_automatic_config_path_from_visible, parse_config, resolve,
};

const V2_STEMS: &[&str] = &[".no-mistakes"];

/// Load the unified `.no-mistakes.yml` (or a recognized legacy config) from
/// `root`, returning a [`NoMistakesConfig`].
///
/// Discovery order:
/// 1. `cli_config` if provided.
/// 2. `.no-mistakes.{yaml,yml,json,jsonc}` in `root`.
/// 3. Empty default.
pub fn load_v2_config(root: &Path, cli_config: Option<&Path>) -> Result<NoMistakesConfig> {
    if cli_config.is_some() {
        return load_v2_config_from_visible(root, cli_config, &[]);
    }

    if let Some((path, source)) = find_by_stems(root, V2_STEMS)? {
        return parse_v2_config(&source, &path);
    }

    Ok(NoMistakesConfig::default())
}

/// Load config while reusing a request's canonical visible-path candidates.
/// Explicit configs deliberately bypass visibility filtering, matching
/// [`load_v2_config`].
#[doc(hidden)]
pub fn load_v2_config_from_visible(
    root: &Path,
    cli_config: Option<&Path>,
    visible_paths: &[PathBuf],
) -> Result<NoMistakesConfig> {
    if let Some(path) = cli_config {
        let resolved = resolve(root, path);
        if !resolved.exists() {
            anyhow::bail!("config file does not exist: {}", resolved.display());
        }
        let source = std::fs::read_to_string(&resolved)?;
        return parse_v2_config(&source, &resolved);
    }

    if let Some(path) = find_automatic_config_path_from_visible(root, V2_STEMS, visible_paths)? {
        let source = std::fs::read_to_string(&path)?;
        return parse_v2_config(&source, &path);
    }

    Ok(NoMistakesConfig::default())
}

#[doc(hidden)]
pub(crate) fn load_v2_config_from_source_store(
    root: &Path,
    cli_config: Option<&Path>,
    visible_paths: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<NoMistakesConfig> {
    let path = effective_v2_config_path_from_visible(root, cli_config, visible_paths)?;
    let Some(path) = path else {
        return Ok(NoMistakesConfig::default());
    };
    let source = sources
        .read_path(&path)
        .map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error))?;
    parse_v2_config(&source, &path)
}

pub(crate) fn effective_v2_config_path_from_visible(
    root: &Path,
    cli_config: Option<&Path>,
    visible_paths: &[PathBuf],
) -> Result<Option<PathBuf>> {
    Ok(if let Some(path) = cli_config {
        let resolved = resolve(root, path);
        if !resolved.exists() {
            anyhow::bail!("config file does not exist: {}", resolved.display());
        }
        Some(resolved)
    } else {
        find_automatic_config_path_from_visible(root, V2_STEMS, visible_paths)?
    })
}

fn parse_v2_config(source: &str, path: &Path) -> Result<NoMistakesConfig> {
    let config = parse_v2_config_quiet(source, path)?;
    emit_v2_deprecation_warnings(&config, path);
    Ok(config)
}

/// Parse and validate v2 config without emitting compatibility warnings.
/// Historical config comparisons use this so one request does not print a
/// warning for a revision that is not its active configuration.
pub(crate) fn parse_v2_config_quiet(source: &str, path: &Path) -> Result<NoMistakesConfig> {
    let config = parse_config::<NoMistakesConfig>(source, path)?;
    validate_v2_config(&config, path)?;
    Ok(config)
}

fn emit_v2_deprecation_warnings(config: &NoMistakesConfig, path: &Path) {
    let path_display = path.display();
    if config.test_plan.playwright.deprecated_dependencies_key {
        eprintln!(
            "warning: {path_display}: `test_plan.playwright.dependencies` is deprecated; \
             rename it to `test_plan.playwright.fullSuiteTriggers`"
        );
    }
    if config.test_plan.vitest.deprecated_dependencies_key {
        eprintln!(
            "warning: {path_display}: `test_plan.vitest.dependencies` is deprecated; \
             rename it to `test_plan.vitest.fullSuiteTriggers`"
        );
    }
}

fn validate_v2_config(config: &NoMistakesConfig, path: &Path) -> Result<()> {
    for (name, project) in &config.projects {
        validate_globs(&project.include, &format!("projects.{name}.include"))?;
        validate_globs(&project.exclude, &format!("projects.{name}.exclude"))?;
    }
    for (index, rule) in config.rules.iter().enumerate() {
        if rule.rule.trim().is_empty() {
            anyhow::bail!("rules[{index}].rule is required");
        }
        validate_globs(&rule.include, &format!("rules[{index}].include"))?;
        validate_globs(&rule.exclude, &format!("rules[{index}].exclude"))?;
    }
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
            for (index, target) in targeted.targets.iter().enumerate() {
                if target.trim().is_empty() {
                    anyhow::bail!("{base}.targets[{index}] must not be blank");
                }
            }
        }
    }
    validate_playwright_selector_wrappers(&config.tests.playwright.selectors.wrappers)?;
    Ok(())
}

fn validate_playwright_selector_wrappers(
    wrappers: &[super::schema::PlaywrightSelectorWrapper],
) -> Result<()> {
    let mut arguments_by_export = BTreeMap::new();
    for (index, wrapper) in wrappers.iter().enumerate() {
        if wrapper.module.trim().is_empty() {
            anyhow::bail!("tests.playwright.selectors.wrappers[{index}].module must not be blank");
        }
        if wrapper.export.trim().is_empty() {
            anyhow::bail!("tests.playwright.selectors.wrappers[{index}].export must not be blank");
        }
        let identity = (wrapper.module.as_str(), wrapper.export.as_str());
        if let Some(previous_argument) =
            arguments_by_export.insert(identity, wrapper.test_id_argument)
        {
            if previous_argument != wrapper.test_id_argument {
                anyhow::bail!(
                    "tests.playwright.selectors.wrappers configures `{}:{}` with conflicting testIdArgument values {previous_argument} and {}",
                    wrapper.module,
                    wrapper.export,
                    wrapper.test_id_argument
                );
            }
        }
    }
    Ok(())
}

fn validate_globs(patterns: &[String], key: &str) -> Result<()> {
    for pattern in patterns {
        GlobBuilder::new(pattern.trim_start_matches("./"))
            .literal_separator(false)
            .build()
            .map_err(|err| anyhow::anyhow!("{key} contains invalid glob `{pattern}`: {err}"))?;
    }
    Ok(())
}

/// Return the directory that contains the effective config for `root`.
pub fn find_config_root(root: &Path) -> PathBuf {
    root.to_path_buf()
}

pub(super) fn find_by_stems(root: &Path, stems: &[&str]) -> Result<Option<(PathBuf, String)>> {
    let Some(path) = find_automatic_config_path(root, stems)? else {
        return Ok(None);
    };
    let source = std::fs::read_to_string(&path)?;
    Ok(Some((path, source)))
}

#[cfg(test)]
mod tests;
