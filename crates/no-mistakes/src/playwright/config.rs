pub use crate::codebase::playwright_facts::Settings;
use anyhow::Result;
use std::path::{Path, PathBuf};

#[path = "config/load.rs"]
mod load;
#[cfg(test)]
pub mod test_support;

pub fn load_settings_from_visible(
    root: &Path,
    cli_config: Option<&Path>,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
    app: Option<String>,
    visible_paths: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Settings> {
    load::load_settings_from_visible(
        root,
        cli_config,
        cli_playwright_configs,
        cli_project,
        app,
        visible_paths,
    )
}

pub fn settings_from_loaded_v2(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
    app: Option<String>,
    visible_paths: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Settings> {
    load::settings_from_loaded_v2(
        root,
        config,
        cli_playwright_configs,
        cli_project,
        app,
        visible_paths,
    )
}

/// Whether `config` opts into per-app Playwright settings resolution at all
/// (a top-level `tests.playwright.*` field, or any rule's `tests.playwright`
/// list). When this is `false`, every Playwright settings call collapses to
/// the same bare-defaults fallback regardless of which frontend app (if any)
/// a caller names, so resolving that app first is wasted work — callers that
/// fan out over multiple frontend apps should check this before doing so.
pub fn has_v2_playwright_settings(config: &crate::config::v2::NoMistakesConfig) -> bool {
    load::has_v2_playwright_settings(config)
}

pub fn has_configured_html_id_selector(settings: &Settings) -> bool {
    use crate::playwright::selectors::HTML_ID_ATTRIBUTE;
    settings
        .selector_attributes
        .iter()
        .any(|attribute| attribute == HTML_ID_ATTRIBUTE)
        || settings
            .component_selector_attributes
            .values()
            .any(|attribute| attribute == HTML_ID_ATTRIBUTE)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
