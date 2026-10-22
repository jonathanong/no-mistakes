use anyhow::Result;
use std::path::{Path, PathBuf};

use super::schema::NoMistakesConfig;
use crate::config::{
    find_automatic_config_path, find_automatic_config_path_from_visible, parse_config, resolve,
};

const V2_STEMS: &[&str] = &[".no-mistakes"];

pub(crate) fn automatic_v2_config_paths(root: &Path) -> Vec<PathBuf> {
    V2_STEMS
        .iter()
        .flat_map(|stem| {
            crate::config::CONFIG_EXTENSIONS
                .iter()
                .map(move |extension| root.join(format!("{stem}.{extension}")))
        })
        .collect()
}

mod check_commands;
mod deprecations;
use deprecations::emit_v2_deprecation_warnings;
mod path_globs;
mod swift_commands;
mod targeted_triggers;

/// Load the unified `.no-mistakes.yml` (or a recognized legacy config) from
/// `root`, returning a [`NoMistakesConfig`].
///
/// Discovery order:
/// 1. `cli_config` if provided.
/// 2. `.no-mistakes.{yaml,yml,json,jsonc}` in `root`.
/// 3. Empty default.
pub fn load_v2_config(root: &Path, cli_config: Option<&Path>) -> Result<NoMistakesConfig> {
    // Bootstrap path: config is loaded before AnalysisSession/SourceStore exist.
    // Prepared requests use `load_v2_config_from_source_store`.
    load_v2_config_with_path(root, cli_config).map(|(config, _)| config)
}

pub(crate) fn load_v2_config_with_path(
    root: &Path,
    cli_config: Option<&Path>,
) -> Result<(NoMistakesConfig, Option<PathBuf>)> {
    if cli_config.is_some() {
        return load_v2_config_with_path_from_visible(root, cli_config, &[]);
    }

    if let Some((path, source)) = find_by_stems(root, V2_STEMS)? {
        return Ok((parse_v2_config(&source, &path)?, Some(path)));
    }

    Ok((NoMistakesConfig::default(), None))
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
    load_v2_config_with_path_from_visible(root, cli_config, visible_paths).map(|(config, _)| config)
}

pub(crate) fn load_v2_config_with_path_from_visible(
    root: &Path,
    cli_config: Option<&Path>,
    visible_paths: &[PathBuf],
) -> Result<(NoMistakesConfig, Option<PathBuf>)> {
    if let Some(path) = cli_config {
        let resolved = resolve(root, path);
        if !resolved.exists() {
            anyhow::bail!("config file does not exist: {}", resolved.display());
        }
        // Snapshot-only bootstrap still precedes SourceStore; prepared analysis
        // uses `load_v2_config_from_source_store`.
        let source = std::fs::read_to_string(&resolved)?;
        return Ok((parse_v2_config(&source, &resolved)?, Some(resolved)));
    }

    if let Some(path) = find_automatic_config_path_from_visible(root, V2_STEMS, visible_paths)? {
        let source = std::fs::read_to_string(&path)?;
        return Ok((parse_v2_config(&source, &path)?, Some(path)));
    }

    Ok((NoMistakesConfig::default(), None))
}

#[doc(hidden)]
pub(crate) fn load_v2_config_from_source_store(
    root: &Path,
    cli_config: Option<&Path>,
    visible_paths: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<NoMistakesConfig> {
    let path = effective_v2_config_path_from_visible(root, cli_config, visible_paths)?;
    load_v2_config_from_selected_source_store(path.as_deref(), sources)
}

pub(crate) fn load_v2_config_from_selected_source_store(
    path: Option<&Path>,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<NoMistakesConfig> {
    let Some(path) = path else {
        return Ok(NoMistakesConfig::default());
    };
    let source = sources
        .read_path(path)
        .map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error))?;
    parse_v2_config(&source, path)
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
    let mut config = parse_config::<NoMistakesConfig>(source, path)?;
    validate_v2_config(&config, path)?;
    super::postgres_project_scope::materialize(&mut config)?;
    config.materialize_analysis_defaults()?;
    for policy in config.tests.vitest.projects.values_mut() {
        if !policy.setup_files.is_empty() {
            policy.declaration_path = Some(crate::codebase::ts_resolver::normalize_path(path));
        }
    }
    config.loaded_config_path =
        super::schema::loaded_path::LoadedConfigPath(Some(path.to_path_buf()));
    Ok(config)
}

fn validate_v2_config(config: &NoMistakesConfig, path: &Path) -> Result<()> {
    for (name, project) in &config.projects {
        path_globs::validate_globs(&project.include, &format!("projects.{name}.include"))?;
        path_globs::validate_globs(&project.exclude, &format!("projects.{name}.exclude"))?;
    }
    for (index, rule) in config.rules.iter().enumerate() {
        if rule.rule.trim().is_empty() {
            anyhow::bail!("rules[{index}].rule is required");
        }
        path_globs::validate_globs(&rule.include, &format!("rules[{index}].include"))?;
        path_globs::validate_globs(&rule.exclude, &format!("rules[{index}].exclude"))?;
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
