use super::merge::{default_config, missing_config_name_error, validate_config_names};
use super::parse::parse_from_path;
use super::types::PlaywrightConfig;
use crate::codebase::ts_source::SourceStore;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub fn load(root: &Path, config_path: &Path) -> Result<PlaywrightConfig> {
    load_with_sources(root, config_path, None)
}

pub(crate) fn load_with_sources(
    root: &Path,
    config_path: &Path,
    sources: Option<&SourceStore>,
) -> Result<PlaywrightConfig> {
    // Resolve a bare config path (one with no parent directory component, like
    // "playwright.config.ts") against `root` so that filesystem operations use
    // an absolute path independent of the process working directory.
    let resolved;
    let config_path = match config_path.parent() {
        Some(p) if !p.as_os_str().is_empty() => config_path,
        _ => {
            resolved = root.join(config_path);
            &resolved
        }
    };

    if !config_path.exists() {
        anyhow::bail!(
            "Playwright config does not exist: {}",
            config_path.display()
        );
    }

    let source = SourceStore::read_prepared_or_open(sources, config_path)
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    let parent = config_path.parent().unwrap_or(root);
    parse_from_path(&source, config_path, parent)
}

pub fn load_many(
    root: &Path,
    config_paths: &[PathBuf],
    config_name_filter: Option<&str>,
) -> Result<PlaywrightConfig> {
    let configs = load_configs(root, config_paths)?;
    select_loaded(root, config_paths, config_name_filter, &configs)
}

pub(crate) fn load_many_with_sources(
    root: &Path,
    config_paths: &[PathBuf],
    config_name_filter: Option<&str>,
    sources: Option<&SourceStore>,
) -> Result<PlaywrightConfig> {
    let configs = load_configs_with_sources(root, config_paths, sources)?;
    select_loaded(root, config_paths, config_name_filter, &configs)
}

pub(crate) fn load_configs(
    root: &Path,
    config_paths: &[PathBuf],
) -> Result<Vec<(PathBuf, PlaywrightConfig)>> {
    load_configs_with_sources(root, config_paths, None)
}

pub(crate) fn load_configs_with_sources(
    root: &Path,
    config_paths: &[PathBuf],
    sources: Option<&SourceStore>,
) -> Result<Vec<(PathBuf, PlaywrightConfig)>> {
    if config_paths.is_empty() {
        return Ok(Vec::new());
    }

    if crate::ast::request_parse_cache_active() {
        // The cached OXC programs are intentionally same-thread. Aggregate analysis
        // loads configs on the owning thread so later runner/check consumers can reuse
        // them; standalone config loading remains parallel.
        config_paths
            .iter()
            .map(|path| load_with_path(root, path, sources))
            .collect()
    } else {
        config_paths
            .par_iter()
            .map(|path| load_with_path(root, path, sources))
            .collect()
    }
}

pub(crate) fn select_loaded(
    root: &Path,
    config_paths: &[PathBuf],
    config_name_filter: Option<&str>,
    loaded: &[(PathBuf, PlaywrightConfig)],
) -> Result<PlaywrightConfig> {
    if config_paths.is_empty() {
        if let Some(name) = config_name_filter {
            anyhow::bail!("--project requires a named Playwright config, but no config was found matching {name}");
        }
        return Ok(default_config(root));
    }

    let configs = config_paths
        .iter()
        .map(|path| {
            let normalized = resolved_config_path(root, path);
            let config = loaded
                .iter()
                .find(|(loaded_path, _)| *loaded_path == normalized)
                .map(|(_, config)| config.clone())
                .ok_or_else(|| {
                    anyhow::anyhow!("Playwright config was not prepared: {}", path.display())
                })?;
            Ok((normalized, config))
        })
        .collect::<Result<Vec<_>>>()?;

    // A single unnamed config has no top-level name to disambiguate, so let
    // `--project` select one of its ordinary Playwright projects directly.
    // Multiple configs must still use unique top-level names.
    let sole_unnamed_project_filter = config_name_filter.filter(|name| {
        configs.len() == 1
            && configs[0].1.name.is_none()
            && configs[0]
                .1
                .projects
                .iter()
                .any(|project| project.name.as_deref() == Some(*name))
    });
    let config_name_filter = config_name_filter.filter(|_| sole_unnamed_project_filter.is_none());
    validate_config_names(&configs, config_name_filter)?;
    match config_name_filter {
        Some(name)
            if !configs
                .iter()
                .any(|(_, config)| config.name.as_deref() == Some(name)) =>
        {
            return Err(missing_config_name_error(name));
        }
        _ => {}
    }

    let mut projects = Vec::new();
    for (_, config) in configs {
        if config_name_filter.is_some_and(|name| config.name.as_deref() != Some(name)) {
            continue;
        }
        projects.extend(config.projects.into_iter().filter(|project| {
            sole_unnamed_project_filter.is_none_or(|name| project.name.as_deref() == Some(name))
        }));
    }

    Ok(PlaywrightConfig {
        name: config_name_filter.map(str::to_string),
        projects,
    })
}

fn load_with_path(
    root: &Path,
    config_path: &Path,
    sources: Option<&SourceStore>,
) -> Result<(PathBuf, PlaywrightConfig)> {
    let config = match sources {
        None => load(root, config_path)?,
        Some(_) => load_with_sources(root, config_path, sources)?,
    };
    Ok((resolved_config_path(root, config_path), config))
}

fn resolved_config_path(root: &Path, config_path: &Path) -> PathBuf {
    let path = if config_path.is_absolute() {
        config_path.to_path_buf()
    } else {
        root.join(config_path)
    };
    crate::codebase::ts_resolver::normalize_path(&path)
}
