use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[path = "config/load.rs"]
mod load;
#[cfg(test)]
pub(crate) mod test_support;

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub frontend_root: String,
    pub playwright_configs: Vec<PathBuf>,
    pub project: Option<String>,
    pub test_include: Vec<String>,
    pub test_exclude: Vec<String>,
    pub ignore_routes: Vec<String>,
    pub rewrites: Vec<crate::config::v2::schema::RewriteRule>,
    pub navigation_helpers: Vec<String>,
    pub selector_wrappers: Vec<crate::config::v2::schema::PlaywrightSelectorWrapper>,
    pub selector_attributes: Vec<String>,
    /// Explicit override for the `getByTestId(...)` attribute, from
    /// `tests.playwright.testIdAttribute`. Use this when the Playwright config's
    /// `testIdAttribute` cannot be read statically (e.g. it is set inside a
    /// helper function).
    pub test_id_attribute_override: Option<String>,
    pub component_selector_attributes: BTreeMap<String, String>,
    pub html_ids: bool,
    pub selector_roots: Vec<String>,
    pub selector_include: Vec<String>,
    pub selector_exclude: Vec<String>,
}

pub(crate) fn load_settings_from_visible(
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

pub(crate) fn settings_from_loaded_v2(
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
pub(crate) fn has_v2_playwright_settings(config: &crate::config::v2::NoMistakesConfig) -> bool {
    load::has_v2_playwright_settings(config)
}

pub(crate) fn has_configured_html_id_selector(settings: &Settings) -> bool {
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

#[cfg(test)]
mod tests;
