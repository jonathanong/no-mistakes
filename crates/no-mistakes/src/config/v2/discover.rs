use anyhow::Result;
use globset::GlobBuilder;
use std::path::{Path, PathBuf};

use super::schema::NoMistakesConfig;
use crate::config::{parse_config, resolve, CONFIG_EXTENSIONS};

const V2_STEMS: &[&str] = &[".no-mistakes"];

/// Load the unified `.no-mistakes.yml` (or a recognized legacy config) from
/// `root`, returning a [`NoMistakesConfig`].
///
/// Discovery order:
/// 1. `cli_config` if provided.
/// 2. `.no-mistakes.{yaml,yml,json,jsonc}` in `root`.
/// 3. Empty default.
pub fn load_v2_config(root: &Path, cli_config: Option<&Path>) -> Result<NoMistakesConfig> {
    if let Some(path) = cli_config {
        let resolved = resolve(root, path);
        if !resolved.exists() {
            anyhow::bail!("config file does not exist: {}", resolved.display());
        }
        let source = std::fs::read_to_string(&resolved)?;
        return parse_v2_config(&source, &resolved);
    }

    if let Some((path, source)) = find_by_stems(root, V2_STEMS)? {
        return parse_v2_config(&source, &path);
    }

    Ok(NoMistakesConfig::default())
}

fn parse_v2_config(source: &str, path: &Path) -> Result<NoMistakesConfig> {
    let config = parse_config::<NoMistakesConfig>(source, path)?;
    validate_v2_config(&config)?;
    emit_v2_deprecation_warnings(&config, path);
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

fn validate_v2_config(config: &NoMistakesConfig) -> Result<()> {
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
    let mut found = Vec::new();
    for stem in stems {
        for ext in CONFIG_EXTENSIONS {
            let path = root.join(format!("{stem}.{ext}"));
            if path.exists() {
                found.push(path);
            }
        }
        if !found.is_empty() {
            break;
        }
    }
    match found.len() {
        0 => Ok(None),
        1 => {
            let path = found.remove(0);
            let source = std::fs::read_to_string(&path)?;
            Ok(Some((path, source)))
        }
        _ => {
            let files = found
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::bail!("multiple config files found under --root: {files}");
        }
    }
}
