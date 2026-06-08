use crate::config::v2::schema::{NoMistakesConfig, PlaywrightTestConfig};
use crate::config::v2::ConfigView;
use crate::config::{resolve, CONFIG_EXTENSIONS};
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

pub(super) fn has_v2_playwright_settings(config: &NoMistakesConfig) -> bool {
    is_v2_playwright_configured(&config.tests.playwright)
}

fn is_v2_playwright_configured(playwright: &PlaywrightTestConfig) -> bool {
    playwright.configs.is_some()
        || !playwright.projects.is_empty()
        || !playwright.test_include.is_empty()
        || !playwright.test_exclude.is_empty()
        || playwright.selectors.html_ids
        || !playwright.selectors.test_ids.is_empty()
        || !playwright.selectors.component_test_ids.is_empty()
        || !playwright.selector_roots.is_empty()
        || !playwright.selector_include.is_empty()
        || !playwright.selector_exclude.is_empty()
        || !playwright.navigation_helpers.is_empty()
        || playwright.frontend_root.is_some()
        || playwright.ignore_routes.is_some()
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
        let file_name = entry.file_name();

        if !is_playwright_config_name(&file_name) {
            continue;
        }

        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        configs.push(path);
    }
    configs.sort();
    Ok(configs)
}

pub(in crate::playwright::config) fn is_playwright_config_name(
    file_name_os: &std::ffi::OsStr,
) -> bool {
    let name = match file_name_os.to_str() {
        Some(name) => name,
        None => return false,
    };

    if !name.starts_with("playwright") || !name.contains(".config.") {
        return false;
    }

    let pos = name.rfind('.').unwrap();
    let extension = &name[pos + 1..];

    PLAYWRIGHT_CONFIG_EXTENSIONS.contains(&extension)
}

pub(super) fn default_selector_attributes() -> Vec<String> {
    DEFAULT_SELECTOR_ATTRIBUTES
        .iter()
        .map(|attribute| attribute.to_string())
        .collect()
}
