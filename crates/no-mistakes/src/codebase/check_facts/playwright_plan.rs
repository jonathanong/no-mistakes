use crate::playwright::playwright_tests::TestPolicy;
use crate::playwright::selectors::SelectorRegexes;
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

mod files;
mod merge;
mod source;

#[derive(Clone, Default)]
pub struct PlaywrightFactPlan {
    files: BTreeMap<PathBuf, PlaywrightFileFactPlan>,
    source_files: Arc<Vec<PathBuf>>,
    source_file_set: Arc<HashSet<PathBuf>>,
    config_files: Arc<HashSet<PathBuf>>,
    source_plans: Vec<PlaywrightSourceFactPlan>,
    test_files_by_project: super::PlaywrightTestFilesByProject,
}

#[derive(Clone)]
pub(crate) struct PlaywrightSourceFactPlan {
    pub(crate) app_source_files: Arc<HashSet<PathBuf>>,
    pub(crate) selector_regexes: Arc<SelectorRegexes>,
    pub(crate) settings: Arc<crate::playwright::config::Settings>,
    pub(crate) visible_files: Arc<HashSet<PathBuf>>,
    pub(crate) scan_html_ids: bool,
    pub(crate) settings_key: PlaywrightSettingsKey,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PlaywrightSettingsKey {
    frontend_root: String,
    playwright_configs: Vec<PathBuf>,
    project: Option<String>,
    test_include: Vec<String>,
    test_exclude: Vec<String>,
    ignore_routes: Vec<String>,
    rewrites: Vec<(String, String)>,
    navigation_helpers: Vec<String>,
    selector_attributes: Vec<String>,
    test_id_attribute_override: Option<String>,
    component_selector_attributes: Vec<(String, String)>,
    html_ids: bool,
    selector_roots: Vec<String>,
    selector_include: Vec<String>,
    selector_exclude: Vec<String>,
}

impl PlaywrightSettingsKey {
    pub(crate) fn new(settings: &crate::playwright::config::Settings) -> Self {
        Self {
            frontend_root: settings.frontend_root.clone(),
            playwright_configs: settings.playwright_configs.clone(),
            project: settings.project.clone(),
            test_include: settings.test_include.clone(),
            test_exclude: settings.test_exclude.clone(),
            ignore_routes: settings.ignore_routes.clone(),
            rewrites: settings
                .rewrites
                .iter()
                .map(|rewrite| (rewrite.source.clone(), rewrite.destination.clone()))
                .collect(),
            navigation_helpers: settings.navigation_helpers.clone(),
            selector_attributes: settings.selector_attributes.clone(),
            test_id_attribute_override: settings.test_id_attribute_override.clone(),
            component_selector_attributes: settings
                .component_selector_attributes
                .iter()
                .map(|(component, attribute)| (component.clone(), attribute.clone()))
                .collect(),
            html_ids: settings.html_ids,
            selector_roots: settings.selector_roots.clone(),
            selector_include: settings.selector_include.clone(),
            selector_exclude: settings.selector_exclude.clone(),
        }
    }
}

pub(crate) struct PlaywrightFactSelection<'a> {
    pub(crate) path: PathBuf,
    pub(crate) navigation_helpers: &'a [String],
    pub(crate) selector_attributes: &'a [String],
    pub(crate) component_selector_attributes: &'a BTreeMap<String, String>,
    pub(crate) html_ids: bool,
    pub(crate) test_id_attributes: &'a [String],
    pub(crate) policy: TestPolicy,
    pub(crate) demands_text_imports: bool,
}

#[derive(Clone)]
pub(crate) struct PlaywrightFileFactPlan {
    variants: BTreeMap<PlaywrightOccurrenceKey, VariantPlan>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct PlaywrightOccurrenceKey {
    pub(crate) navigation_helpers: Vec<String>,
    pub(crate) selector_attributes: Vec<String>,
    pub(crate) component_selector_attributes: BTreeMap<String, String>,
    pub(crate) html_ids: bool,
    pub(crate) test_id_attributes: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct VariantPlan {
    pub(crate) selector_regexes: Arc<SelectorRegexes>,
    policies: Vec<TestPolicy>,
}

#[cfg(test)]
mod tests;
