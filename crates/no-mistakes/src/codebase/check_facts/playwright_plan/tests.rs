use super::{
    BTreeMap, PlaywrightFactPlan, PlaywrightFactSelection, PlaywrightFileFactPlan,
    PlaywrightOccurrenceKey,
};
use crate::playwright::playwright_tests::TestPolicy;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl PlaywrightFactPlan {
    pub(crate) fn from_settings(
        root: &Path,
        settings: crate::playwright::config::Settings,
        test_id_attributes_by_path: HashMap<PathBuf, Vec<String>>,
        scan_html_ids: bool,
        snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    ) -> anyhow::Result<Self> {
        let navigation_helpers = settings.navigation_helpers.clone();
        let selector_attributes = settings.selector_attributes.clone();
        let component_selector_attributes = settings.component_selector_attributes.clone();
        let html_ids = settings.html_ids;
        let mut plan = Self::default();
        plan.add_source_settings(root, settings, scan_html_ids, snapshot)?;
        for (path, test_id_attributes) in test_id_attributes_by_path {
            plan.add_file(PlaywrightFactSelection {
                path,
                navigation_helpers: &navigation_helpers,
                selector_attributes: &selector_attributes,
                component_selector_attributes: &component_selector_attributes,
                html_ids,
                test_id_attributes: &test_id_attributes,
                policy: TestPolicy::default(),
                demands_text_imports: true,
            });
        }
        Ok(plan)
    }

    pub(crate) fn set_app_source_files(&mut self, files: impl IntoIterator<Item = PathBuf>) {
        let files = Arc::new(
            files
                .into_iter()
                .map(|path| crate::codebase::ts_resolver::normalize_path(&path))
                .collect::<HashSet<_>>(),
        );
        for plan in &mut self.source_plans {
            plan.app_source_files = Arc::clone(&files);
        }
    }
}

impl PlaywrightFileFactPlan {
    pub(crate) fn merged_test_id_attributes(&self) -> Vec<String> {
        self.variants
            .keys()
            .flat_map(|key| key.test_id_attributes.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub(crate) fn selector_extraction_count(&self) -> usize {
        self.variants.len()
    }
}

#[test]
fn occurrence_key_sorts_and_deduplicates_sequence_fields() {
    let key = PlaywrightOccurrenceKey::new(
        &["goB".to_string(), "goA".to_string(), "goB".to_string()],
        &["data-b".to_string(), "data-a".to_string()],
        &BTreeMap::from([
            ("propB".to_string(), "data-b".to_string()),
            ("propA".to_string(), "data-a".to_string()),
        ]),
        true,
        &[
            "data-b".to_string(),
            "data-a".to_string(),
            "data-b".to_string(),
        ],
    );

    assert_eq!(key.navigation_helpers, ["goA", "goB"]);
    assert_eq!(key.selector_attributes, ["data-a", "data-b"]);
    assert_eq!(key.test_id_attributes, ["data-a", "data-b"]);
    assert_eq!(key.component_selector_attributes["propA"], "data-a");
    assert!(key.html_ids);
}
