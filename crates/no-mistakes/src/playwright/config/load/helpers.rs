use super::{FileConfig, OneOrMany, RootConfig};
use crate::config::v2::schema::{NoMistakesConfig, PlaywrightTestConfig, ProjectType};
use crate::config::v2::ConfigView;
use crate::config::{parse_config, resolve, CONFIG_EXTENSIONS};
use anyhow::Result;
use std::path::{Path, PathBuf};

const DEFAULT_SELECTOR_ATTRIBUTES: &[&str] = &["data-testid", "data-pw"];
const PLAYWRIGHT_CONFIG_EXTENSIONS: &[&str] = &["ts", "mts", "cts", "js", "mjs", "cjs"];

pub(super) fn playwright_configs_from_v2(
    root: &Path,
    view: &ConfigView,
    cli_playwright_configs: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    if !cli_playwright_configs.is_empty() {
        return Ok(cli_playwright_configs
            .iter()
            .map(|path| resolve(root, path))
            .collect());
    }
    if let Some(paths) = view.playwright_configs() {
        return Ok(paths
            .iter()
            .map(|path| resolve(root, Path::new(path)))
            .collect());
    }
    find_default_playwright_configs(root)
}

pub(super) fn playwright_configs_from_legacy(
    root: &Path,
    file_config: &FileConfig,
    cli_playwright_configs: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    if !cli_playwright_configs.is_empty() {
        return Ok(cli_playwright_configs
            .iter()
            .map(|path| resolve(root, path))
            .collect());
    }
    if let Some(paths) = file_config.playwright_config.as_ref() {
        return Ok(paths
            .values()
            .iter()
            .map(|path| resolve(root, Path::new(path)))
            .collect());
    }
    find_default_playwright_configs(root)
}

impl OneOrMany {
    pub(in crate::playwright::config) fn values(&self) -> Vec<String> {
        match self {
            OneOrMany::One(value) => vec![value.clone()],
            OneOrMany::Many(values) => values.clone(),
        }
    }
}

pub(super) fn parse_legacy_playwright_config(source: &str, path: &Path) -> Result<FileConfig> {
    let root_config: RootConfig = parse_config(source, path)?;
    Ok(root_config
        .playwright_ast_coverage
        .unwrap_or(root_config.legacy))
}

pub(super) fn has_v2_playwright_settings(config: &NoMistakesConfig) -> bool {
    let playwright = &config.tests.playwright;
    has_nextjs_project(config) || is_v2_playwright_configured(playwright)
}

fn has_nextjs_project(config: &NoMistakesConfig) -> bool {
    config
        .projects
        .values()
        .any(|project| project.type_ == Some(ProjectType::Nextjs))
}

fn is_v2_playwright_configured(playwright: &PlaywrightTestConfig) -> bool {
    playwright.configs.is_some()
        || !playwright.projects.is_empty()
        || playwright.selectors.html_ids
        || !playwright.selectors.test_ids.is_empty()
        || !playwright.selectors.component_test_ids.is_empty()
        || !playwright.selector_roots.is_empty()
        || !playwright.selector_exclude.is_empty()
}

pub(super) fn is_legacy_playwright_configured(config: &FileConfig) -> bool {
    config.frontend_root.is_some()
        || config.playwright_config.is_some()
        || !config.test_include.is_empty()
        || !config.test_exclude.is_empty()
        || !config.ignore_routes.is_empty()
        || !config.navigation_helpers.is_empty()
        || config.selector_attributes.is_some()
        || !config.component_selector_attributes.is_empty()
        || config.html_ids
        || config.selector_roots.is_some()
        || !config.selector_include.is_empty()
        || !config.selector_exclude.is_empty()
}

pub(super) fn is_v2_config_path(path: &Path) -> bool {
    path.file_stem().and_then(|stem| stem.to_str()) == Some(".no-mistakes")
}

pub(super) fn find_by_stems(root: &Path, stems: &[&str]) -> Result<Option<(PathBuf, String)>> {
    let mut found = Vec::new();
    for stem in stems {
        for extension in CONFIG_EXTENSIONS {
            let path = root.join(format!("{stem}.{extension}"));
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
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::bail!("multiple config files found under --root: {files}");
        }
    }
}

pub(super) fn find_default_playwright_configs(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut configs = Vec::new();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || !is_playwright_config_name(&path) {
            continue;
        }
        configs.push(path);
    }
    configs.sort();
    Ok(configs)
}

pub(in crate::playwright::config) fn is_playwright_config_name(path: &Path) -> bool {
    let name = match path.file_name().and_then(|name| name.to_str()) {
        Some(name) => name,
        None => return false,
    };
    let extension = match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) => extension,
        None => return false,
    };

    name.starts_with("playwright")
        && name.contains(".config.")
        && PLAYWRIGHT_CONFIG_EXTENSIONS.contains(&extension)
}

pub(super) fn default_selector_attributes() -> Vec<String> {
    DEFAULT_SELECTOR_ATTRIBUTES
        .iter()
        .map(|attribute| attribute.to_string())
        .collect()
}
