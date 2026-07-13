use anyhow::{Context, Result};
use jsonc_parser::ParseOptions;
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) const CONFIG_EXTENSIONS: &[&str] = &["yaml", "yml", "json", "jsonc"];

pub fn load_config<T: DeserializeOwned + Default>(
    root: &Path,
    cli_config: Option<&Path>,
    stems: &[&str],
) -> Result<T> {
    let Some(path) = find_config_path(root, cli_config, stems)? else {
        return Ok(T::default());
    };

    let source = std::fs::read_to_string(&path)?;
    parse_config(&source, &path)
}

/// Load a legacy config while reusing the caller's request-scoped visible
/// paths for automatic discovery. An explicit CLI path remains authoritative,
/// including when Git ignores that path.
pub fn load_config_from_visible<T: DeserializeOwned + Default>(
    root: &Path,
    cli_config: Option<&Path>,
    stems: &[&str],
    visible_paths: &[PathBuf],
) -> Result<T> {
    let path = if let Some(path) = cli_config {
        let config_path = resolve(root, path);
        if !config_path.exists() {
            anyhow::bail!("config file does not exist: {}", config_path.display());
        }
        Some(config_path)
    } else {
        find_automatic_config_path_from_visible(root, stems, visible_paths)?
    };
    let Some(path) = path else {
        return Ok(T::default());
    };

    let source = std::fs::read_to_string(&path)?;
    parse_config(&source, &path)
}

fn find_config_path(
    root: &Path,
    cli_config: Option<&Path>,
    stems: &[&str],
) -> Result<Option<PathBuf>> {
    if let Some(path) = cli_config {
        let config_path = resolve(root, path);
        if !config_path.exists() {
            anyhow::bail!("config file does not exist: {}", config_path.display());
        }
        return Ok(Some(config_path));
    }

    find_automatic_config_path(root, stems)
}

pub(crate) fn find_automatic_config_path(root: &Path, stems: &[&str]) -> Result<Option<PathBuf>> {
    if !stems.iter().any(|stem| {
        CONFIG_EXTENSIONS
            .iter()
            .any(|extension| root.join(format!("{stem}.{extension}")).exists())
    }) {
        return Ok(None);
    }
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(root);
    find_automatic_config_path_from_visible(root, stems, &visible_paths)
}

pub(crate) fn find_automatic_config_path_from_visible(
    root: &Path,
    stems: &[&str],
    visible_paths: &[PathBuf],
) -> Result<Option<PathBuf>> {
    let visible_paths: HashSet<PathBuf> = visible_paths
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect();
    for stem in stems {
        let mut found_configs = Vec::new();
        for extension in CONFIG_EXTENSIONS {
            let path = root.join(format!("{stem}.{extension}"));
            if path.exists()
                && visible_paths.contains(&crate::codebase::ts_resolver::normalize_path(&path))
            {
                found_configs.push(path);
            }
        }
        match found_configs.len() {
            0 => {}
            1 => return Ok(found_configs.pop()),
            _ => return multiple_configs_error(&found_configs),
        }
    }
    Ok(None)
}

fn multiple_configs_error<T>(found_configs: &[PathBuf]) -> Result<T> {
    let files = found_configs
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    anyhow::bail!("multiple config files found under --root: {files}");
}

pub fn parse_config<T: DeserializeOwned>(source: &str, path: &Path) -> Result<T> {
    let extension = path.extension().and_then(|e| e.to_str());
    match extension {
        Some("yaml" | "yml") => serde_yaml::from_str(source)
            .with_context(|| format!("failed to parse {}", path.display())),
        Some("json") => serde_json::from_str(source)
            .with_context(|| format!("failed to parse {}", path.display())),
        Some("jsonc") => serde_json::from_value(jsonc_parser::parse_to_serde_value(
            source,
            &jsonc_parse_options(),
        )?)
        .with_context(|| format!("failed to parse {}", path.display())),
        Some(extension) => anyhow::bail!(
            "unsupported config file extension .{extension}; supported extensions are .yaml, .yml, .json, and .jsonc"
        ),
        None => anyhow::bail!(
            "unsupported config file without extension; supported extensions are .yaml, .yml, .json, and .jsonc"
        ),
    }
}

fn jsonc_parse_options() -> ParseOptions {
    ParseOptions {
        allow_comments: true,
        allow_loose_object_property_names: false,
        allow_trailing_commas: true,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    }
}

pub fn resolve(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

pub mod v2;

#[cfg(test)]
mod tests;
