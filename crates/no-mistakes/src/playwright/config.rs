use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[path = "config/load.rs"]
mod load;

#[derive(Clone)]
pub struct Settings {
    pub frontend_root: String,
    pub playwright_configs: Vec<PathBuf>,
    pub project: Option<String>,
    pub test_include: Vec<String>,
    pub test_exclude: Vec<String>,
    pub ignore_routes: Vec<String>,
    pub navigation_helpers: Vec<String>,
    pub selector_attributes: Vec<String>,
    pub component_selector_attributes: BTreeMap<String, String>,
    pub html_ids: bool,
    pub selector_roots: Vec<String>,
    pub selector_include: Vec<String>,
    pub selector_exclude: Vec<String>,
}

pub fn load_settings(
    root: &Path,
    cli_config: Option<&Path>,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
) -> Result<Settings> {
    load::load_settings(root, cli_config, cli_playwright_configs, cli_project)
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
