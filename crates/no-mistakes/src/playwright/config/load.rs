use super::Settings;
use crate::config::find_automatic_config_path_from_visible;
use crate::config::resolve;
use crate::config::v2::load_v2_config;
use anyhow::Result;
use std::path::{Path, PathBuf};

#[path = "load/helpers.rs"]
pub(super) mod helpers;
use helpers::{default_selector_attributes, has_v2_playwright_settings};

#[path = "load/loaded_v2.rs"]
mod loaded_v2;

const V2_STEMS: &[&str] = &[".no-mistakes"];

pub(super) fn load_settings_from_visible(
    root: &Path,
    cli_config: Option<&Path>,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
    visible_paths: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Settings> {
    if let Some(path) = cli_config {
        return load_explicit(
            root,
            path,
            cli_playwright_configs,
            cli_project,
            visible_paths,
        );
    }
    if let Some(settings) = load_discovered_v2(
        root,
        cli_playwright_configs,
        cli_project.clone(),
        visible_paths,
    )? {
        return Ok(settings);
    }
    settings_from_defaults(root, cli_playwright_configs, cli_project, visible_paths)
}

fn load_explicit(
    root: &Path,
    path: &Path,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
    visible_paths: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Settings> {
    let resolved = resolve(root, path);
    if !resolved.exists() {
        anyhow::bail!("config file does not exist: {}", resolved.display());
    }
    let v2 = load_v2_config(root, Some(&resolved))?;
    if has_v2_playwright_settings(&v2) {
        return loaded_v2::settings_from_v2(
            root,
            &v2,
            cli_playwright_configs,
            cli_project,
            visible_paths,
        );
    }
    settings_from_defaults(root, cli_playwright_configs, cli_project, visible_paths)
}

fn load_discovered_v2(
    root: &Path,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
    visible_paths: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Option<Settings>> {
    let paths = visible_paths.paths_for(root);
    let Some(path) = find_automatic_config_path_from_visible(root, V2_STEMS, &paths)? else {
        return Ok(None);
    };
    let v2 = load_v2_config(root, Some(&path))?;
    if has_v2_playwright_settings(&v2) {
        return loaded_v2::settings_from_v2(
            root,
            &v2,
            cli_playwright_configs,
            cli_project,
            visible_paths,
        )
        .map(Some);
    }
    Ok(None)
}

pub(super) fn settings_from_loaded_v2(
    root: &Path,
    config: &crate::config::v2::schema::NoMistakesConfig,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
    visible_paths: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Settings> {
    loaded_v2::settings_from_loaded_v2(
        root,
        config,
        cli_playwright_configs,
        cli_project,
        visible_paths,
    )
}

fn settings_from_defaults(
    root: &Path,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
    visible_paths: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Settings> {
    let playwright_configs = if cli_playwright_configs.is_empty() {
        helpers::find_default_playwright_configs_from_visible(root, &visible_paths.paths_for(root))?
    } else {
        cli_playwright_configs
            .iter()
            .map(|path| resolve(root, path))
            .collect()
    };
    let frontend_root = "app".to_string();
    Ok(Settings {
        frontend_root: frontend_root.clone(),
        playwright_configs,
        project: cli_project,
        test_include: Vec::new(),
        test_exclude: Vec::new(),
        ignore_routes: Vec::new(),
        rewrites: Vec::new(),
        navigation_helpers: Vec::new(),
        selector_attributes: default_selector_attributes(),
        test_id_attribute_override: None,
        component_selector_attributes: Default::default(),
        html_ids: false,
        selector_roots: vec![frontend_root],
        selector_include: Vec::new(),
        selector_exclude: Vec::new(),
    })
}
