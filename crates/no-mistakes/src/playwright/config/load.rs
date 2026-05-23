use super::Settings;
use crate::config::resolve;
use crate::config::v2::schema::NoMistakesConfig;
use crate::config::v2::{load_v2_config, ConfigView};
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[path = "load/helpers.rs"]
pub(super) mod helpers;
use helpers::{
    default_selector_attributes, find_by_stems, has_v2_playwright_settings,
    is_legacy_playwright_configured, is_v2_config_path, parse_legacy_playwright_config,
    playwright_configs_from_legacy, playwright_configs_from_v2,
};

const V2_STEMS: &[&str] = &[".no-mistakes"];
const LEGACY_PLAYWRIGHT_STEMS: &[&str] = &[".playwright-ast-coverage"];
const DEFAULT_FRONTEND_ROOT: &str = "app";

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct RootConfig {
    #[serde(flatten)]
    pub(super) legacy: FileConfig,
    pub(super) playwright_ast_coverage: Option<FileConfig>,
}

#[derive(Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct FileConfig {
    pub(super) frontend_root: Option<String>,
    pub(super) playwright_config: Option<OneOrMany>,
    pub(super) test_include: Vec<String>,
    pub(super) test_exclude: Vec<String>,
    pub(super) ignore_routes: Vec<String>,
    pub(super) navigation_helpers: Vec<String>,
    pub(super) selector_attributes: Option<Vec<String>>,
    pub(super) component_selector_attributes: BTreeMap<String, String>,
    pub(super) html_ids: bool,
    pub(super) selector_roots: Option<Vec<String>>,
    pub(super) selector_include: Vec<String>,
    pub(super) selector_exclude: Vec<String>,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub(super) enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

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
    if let Some((path, source)) = find_by_stems(root, LEGACY_PLAYWRIGHT_STEMS)? {
        let legacy = parse_legacy_playwright_config(&source, &path)?;
        return settings_from_legacy_file_config(root, legacy, cli_playwright_configs, cli_project);
    }
    settings_from_legacy_file_config(
        root,
        FileConfig::default(),
        cli_playwright_configs,
        cli_project,
    )
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
    if is_v2_config_path(&resolved) {
        let v2 = load_v2_config(root, Some(&resolved))?;
        if has_v2_playwright_settings(&v2) {
            return settings_from_v2(root, &v2, cli_playwright_configs, cli_project);
        }
    }
    let source = std::fs::read_to_string(&resolved)?;
    let legacy = parse_legacy_playwright_config(&source, &resolved)?;
    settings_from_legacy_file_config(root, legacy, cli_playwright_configs, cli_project)
}

fn load_discovered_v2(
    root: &Path,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
) -> Result<Option<Settings>> {
    let Some((path, source)) = find_by_stems(root, V2_STEMS)? else {
        return Ok(None);
    };
    let v2 = load_v2_config(root, Some(&path))?;
    if has_v2_playwright_settings(&v2) {
        return settings_from_v2(root, &v2, cli_playwright_configs, cli_project).map(Some);
    }
    let legacy = parse_legacy_playwright_config(&source, &path)?;
    if is_legacy_playwright_configured(&legacy) {
        return settings_from_legacy_file_config(root, legacy, cli_playwright_configs, cli_project)
            .map(Some);
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
    let frontend_root = view.nextjs_root().to_string();
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
    Ok(Settings {
        frontend_root,
        playwright_configs,
        project: cli_project,
        test_include: Vec::new(),
        test_exclude: Vec::new(),
        ignore_routes: Vec::new(),
        navigation_helpers: Vec::new(),
        selector_attributes,
        component_selector_attributes: playwright.selectors.component_test_ids.clone(),
        html_ids: playwright.selectors.html_ids,
        selector_roots,
        selector_include: Vec::new(),
        selector_exclude: playwright.selector_exclude.clone(),
    })
}

fn settings_from_legacy_file_config(
    root: &Path,
    file_config: FileConfig,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
) -> Result<Settings> {
    let playwright_configs =
        playwright_configs_from_legacy(root, &file_config, cli_playwright_configs)?;
    let frontend_root = file_config
        .frontend_root
        .unwrap_or_else(|| DEFAULT_FRONTEND_ROOT.to_string());
    let selector_roots = file_config
        .selector_roots
        .unwrap_or_else(|| vec![frontend_root.clone()]);
    Ok(Settings {
        frontend_root,
        playwright_configs,
        project: cli_project,
        test_include: file_config.test_include,
        test_exclude: file_config.test_exclude,
        ignore_routes: file_config.ignore_routes,
        navigation_helpers: file_config.navigation_helpers,
        selector_attributes: file_config
            .selector_attributes
            .unwrap_or_else(default_selector_attributes),
        component_selector_attributes: file_config.component_selector_attributes,
        html_ids: file_config.html_ids,
        selector_roots,
        selector_include: file_config.selector_include,
        selector_exclude: file_config.selector_exclude,
    })
}
