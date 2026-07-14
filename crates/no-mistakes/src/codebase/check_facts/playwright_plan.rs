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
        fn normalized<T: Clone + Ord>(values: &[T]) -> Vec<T> {
            let mut values = values.to_vec();
            values.sort();
            values.dedup();
            values
        }

        Self {
            frontend_root: settings.frontend_root.clone(),
            playwright_configs: settings.playwright_configs.clone(),
            project: settings.project.clone(),
            test_include: normalized(&settings.test_include),
            test_exclude: normalized(&settings.test_exclude),
            ignore_routes: normalized(&settings.ignore_routes),
            rewrites: settings
                .rewrites
                .iter()
                .map(|rewrite| (rewrite.source.clone(), rewrite.destination.clone()))
                .collect(),
            navigation_helpers: normalized(&settings.navigation_helpers),
            selector_attributes: normalized(&settings.selector_attributes),
            test_id_attribute_override: settings.test_id_attribute_override.clone(),
            component_selector_attributes: settings
                .component_selector_attributes
                .iter()
                .map(|(component, attribute)| (component.clone(), attribute.clone()))
                .collect(),
            html_ids: settings.html_ids,
            selector_roots: normalized(&settings.selector_roots),
            selector_include: normalized(&settings.selector_include),
            selector_exclude: normalized(&settings.selector_exclude),
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
