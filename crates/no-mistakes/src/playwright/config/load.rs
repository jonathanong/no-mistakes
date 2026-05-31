use super::Settings;
use crate::config::resolve;
use crate::config::v2::schema::NoMistakesConfig;
use crate::config::v2::{load_v2_config, ConfigView};
use anyhow::Result;
use std::path::{Path, PathBuf};

#[path = "load/helpers.rs"]
pub(super) mod helpers;
use helpers::{
    default_selector_attributes, find_by_stems, has_v2_playwright_settings,
    playwright_configs_from_v2,
};

const V2_STEMS: &[&str] = &[".no-mistakes"];

pub(super) fn load_settings(
    root: &Path,
    cli_config: Option<&Path>,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
) -> Result<Settings> {
    if let Some(path) = cli_config {
        return load_explicit(root, path, cli_playwright_configs, cli_project);
    }
    if let Some(settings) = load_discovered_v2(root, cli_playwright_configs, cli_project.clone())? {
        return Ok(settings);
    }
    settings_from_defaults(root, cli_playwright_configs, cli_project)
}

fn load_explicit(
    root: &Path,
    path: &Path,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
) -> Result<Settings> {
    let resolved = resolve(root, path);
    if !resolved.exists() {
        anyhow::bail!("config file does not exist: {}", resolved.display());
    }
    let v2 = load_v2_config(root, Some(&resolved))?;
    if has_v2_playwright_settings(&v2) {
        return settings_from_v2(root, &v2, cli_playwright_configs, cli_project);
    }
    settings_from_defaults(root, cli_playwright_configs, cli_project)
}

fn load_discovered_v2(
    root: &Path,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
) -> Result<Option<Settings>> {
    let Some((path, _source)) = find_by_stems(root, V2_STEMS)? else {
        return Ok(None);
    };
    let v2 = load_v2_config(root, Some(&path))?;
    if has_v2_playwright_settings(&v2) {
        return settings_from_v2(root, &v2, cli_playwright_configs, cli_project).map(Some);
    }
    Ok(None)
}

fn settings_from_v2(
    root: &Path,
    config: &NoMistakesConfig,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
) -> Result<Settings> {
    let view = ConfigView::new(config);
    let playwright = &config.tests.playwright;
    let frontend_root = playwright
        .frontend_root
        .as_deref()
        .unwrap_or_else(|| view.nextjs_root())
        .to_string();
    let playwright_configs = playwright_configs_from_v2(root, &view, cli_playwright_configs)?;
    let selector_attributes = if view.test_id_attributes().is_empty() {
        default_selector_attributes()
    } else {
        view.test_id_attributes().to_vec()
    };
    let selector_roots = if view.selector_roots().is_empty() {
        vec![frontend_root.clone()]
    } else {
        view.selector_roots().to_vec()
    };
    let ignore_routes = playwright.ignore_routes.clone().unwrap_or_default();
    let rewrites = view.nextjs_rewrites().to_vec();
    Ok(Settings {
        frontend_root,
        playwright_configs,
        project: cli_project,
        test_include: playwright.test_include.clone(),
        test_exclude: playwright.test_exclude.clone(),
        ignore_routes,
        rewrites,
        navigation_helpers: playwright.navigation_helpers.clone(),
        selector_attributes,
        component_selector_attributes: playwright.selectors.component_test_ids.clone(),
        html_ids: playwright.selectors.html_ids,
        selector_roots,
        selector_include: playwright.selector_include.clone(),
        selector_exclude: playwright.selector_exclude.clone(),
    })
}

fn settings_from_defaults(
    root: &Path,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
) -> Result<Settings> {
    let playwright_configs = if cli_playwright_configs.is_empty() {
        helpers::find_default_playwright_configs(root)?
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
        component_selector_attributes: Default::default(),
        html_ids: false,
        selector_roots: vec![frontend_root],
        selector_include: Vec::new(),
        selector_exclude: Vec::new(),
    })
}
