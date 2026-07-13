use super::merge::default_test_match;
use super::parse::parse_from_path;
use super::types::{PlaywrightConfig, TestProject};
use anyhow::Result;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn load(root: &Path, config_path: &Path) -> Result<PlaywrightConfig> {
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

    let source = std::fs::read_to_string(config_path)?;
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

pub(crate) fn load_configs(
    root: &Path,
    config_paths: &[PathBuf],
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
            .map(|path| load_with_path(root, path))
            .collect()
    } else {
        config_paths
            .par_iter()
            .map(|path| load_with_path(root, path))
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

fn load_with_path(root: &Path, config_path: &Path) -> Result<(PathBuf, PlaywrightConfig)> {
    Ok((
        resolved_config_path(root, config_path),
        load(root, config_path)?,
    ))
}

fn resolved_config_path(root: &Path, config_path: &Path) -> PathBuf {
    let path = if config_path.is_absolute() {
        config_path.to_path_buf()
    } else {
        root.join(config_path)
    };
    crate::codebase::ts_resolver::normalize_path(&path)
}

fn missing_config_name_error(name: &str) -> anyhow::Error {
    anyhow::Error::msg(format!("no Playwright config found with name {name}"))
}

fn default_config(root: &Path) -> PlaywrightConfig {
    PlaywrightConfig {
        name: None,
        projects: vec![TestProject {
            name: None,
            config_dir: root.to_path_buf(),
            test_dir: ".".to_string(),
            test_match: default_test_match(),
            test_ignore: Vec::new(),
            base_url: None,
            // Synthesized fallback config: the attribute was not read from a real
            // Playwright config, so leave it `None` to defer to `selectors.testIds`.
            test_id_attribute: None,
        }],
    }
}

fn validate_config_names(
    configs: &[(PathBuf, PlaywrightConfig)],
    config_name_filter: Option<&str>,
) -> Result<()> {
    if configs.len() <= 1 && config_name_filter.is_none() {
        return Ok(());
    }

    let mut seen = BTreeMap::new();
    for (path, config) in configs {
        let Some(name) = config.name.as_deref() else {
            anyhow::bail!(
                "Playwright config {} must define top-level name when multiple configs are analyzed or --project is used",
                path.display()
            );
        };
        if let Some(previous) = seen.insert(name.to_string(), path.display().to_string()) {
            anyhow::bail!(
                "Playwright config name {name} is duplicated by {} and {}",
                previous,
                path.display()
            );
        }
    }
    Ok(())
}
