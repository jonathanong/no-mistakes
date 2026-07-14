use crate::config::resolve;
use crate::config::v2::schema::{NoMistakesConfig, PlaywrightTestConfig};
use crate::config::v2::ConfigView;
use anyhow::Result;
use std::path::{Path, PathBuf};

const DEFAULT_SELECTOR_ATTRIBUTES: &[&str] = &["data-testid", "data-pw"];
const PLAYWRIGHT_CONFIG_EXTENSIONS: &[&str] = &["ts", "mts", "cts", "js", "mjs", "cjs"];

pub(super) fn playwright_configs_from_v2(
    root: &Path,
    view: &ConfigView,
    cli_playwright_configs: &[PathBuf],
    visible_paths: &crate::playwright::fsutil::VisiblePathSnapshot,
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
    find_default_playwright_configs_from_snapshot(root, visible_paths)
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
        || !playwright.selectors.wrappers.is_empty()
        || !playwright.selector_roots.is_empty()
        || !playwright.selector_include.is_empty()
        || !playwright.selector_exclude.is_empty()
        || !playwright.navigation_helpers.is_empty()
        || playwright.frontend_root.is_some()
        || playwright.ignore_routes.is_some()
}

pub(super) fn find_default_playwright_configs_from_snapshot(
    root: &Path,
    snapshot: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Vec<PathBuf>> {
    let sources = snapshot.source_store_for(root);
    find_default_playwright_configs(root, &sources.inventory().paths(), sources.inventory())
}

fn find_default_playwright_configs(
    root: &Path,
    visible_paths: &[PathBuf],
    inventory: &crate::codebase::ts_source::FileInventory,
) -> Result<Vec<PathBuf>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut configs: Vec<PathBuf> = visible_paths
        .iter()
        .filter(|path| {
            crate::codebase::ts_resolver::normalize_path(path).parent() == Some(root.as_path())
        })
        .filter(|path| path.file_name().is_some_and(is_playwright_config_name))
        // Follow a visible symlink, preserving the existing config-file policy.
        .filter(|path| {
            inventory
                .classification_for_path(path)
                .is_some_and(crate::codebase::ts_source::FileClassification::target_is_file)
        })
        .cloned()
        .collect();
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
