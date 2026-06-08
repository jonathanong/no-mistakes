use super::merge::default_test_match;
use super::merge::DEFAULT_TEST_ID_ATTRIBUTE;
use super::parse::parse_from_path;
use super::types::{PlaywrightConfig, TestProject};
use anyhow::Result;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn load(root: &Path, config_path: &Path) -> Result<PlaywrightConfig> {
    if !config_path.exists() {
        anyhow::bail!(
            "Playwright config does not exist: {}",
            config_path.display()
        );
    }

    let source = std::fs::read_to_string(config_path)?;
    parse_from_path(&source, config_path, config_path.parent().unwrap_or(root))
}

pub fn load_many(
    root: &Path,
    config_paths: &[PathBuf],
    config_name_filter: Option<&str>,
) -> Result<PlaywrightConfig> {
    if config_paths.is_empty() {
        if let Some(name) = config_name_filter {
            anyhow::bail!("--project requires a named Playwright config, but no config was found matching {name}");
        }
        return Ok(default_config(root));
    }

    let configs: Vec<(&PathBuf, PlaywrightConfig)> = config_paths
        .par_iter()
        .map(|config_path| {
            let config = load(root, config_path)?;
            Ok((config_path, config))
        })
        .collect::<Result<Vec<_>>>()?;

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
        projects.extend(config.projects);
    }

    Ok(PlaywrightConfig {
        name: config_name_filter.map(str::to_string),
        projects,
    })
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
            test_id_attribute: DEFAULT_TEST_ID_ATTRIBUTE.to_string(),
        }],
    }
}

fn validate_config_names(
    configs: &[(&PathBuf, PlaywrightConfig)],
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
