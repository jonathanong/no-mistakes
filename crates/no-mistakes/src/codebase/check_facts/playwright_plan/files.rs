use super::merge::merge_sorted;
use super::{
    PlaywrightFactPlan, PlaywrightFactSelection, PlaywrightFileFactPlan, PlaywrightOccurrenceKey,
    VariantPlan,
};
use crate::playwright::playwright_tests::TestOccurrenceScope;
use crate::playwright::selectors::compile_selector_regexes_with_html_ids;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl PlaywrightFactPlan {
    pub(crate) fn add_file(&mut self, selection: PlaywrightFactSelection<'_>) {
        let entry = self
            .files
            .entry(selection.path.clone())
            .or_insert_with(PlaywrightFileFactPlan::empty);
        entry.merge(&selection);
    }

    pub(crate) fn file(&self, path: &Path) -> Option<&PlaywrightFileFactPlan> {
        self.files.get(path)
    }

    pub(crate) fn paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.files.keys()
    }

    pub(crate) fn demands_text_imports(
        &self,
        facts: &BTreeMap<PathBuf, &super::super::PlaywrightTestFacts>,
    ) -> bool {
        facts.iter().any(|(path, facts)| {
            self.file(path).is_some_and(|plan| {
                facts.common().text_locators.iter().any(|occurrence| {
                    occurrence.scope != TestOccurrenceScope::TeardownHook
                        && plan
                            .variants
                            .values()
                            .flat_map(|variant| variant.policies.iter())
                            .any(|policy| policy.allows(occurrence.status))
                })
            })
        })
    }
}

impl PlaywrightFileFactPlan {
    pub(super) fn empty() -> Self {
        Self {
            variants: BTreeMap::new(),
        }
    }

    fn merge(&mut self, selection: &PlaywrightFactSelection<'_>) {
        let key = PlaywrightOccurrenceKey::new(
            selection.navigation_helpers,
            selection.selector_wrappers,
            selection.selector_attributes,
            selection.component_selector_attributes,
            selection.html_ids,
            selection.test_id_attributes,
        );
        let variant = self
            .variants
            .entry(key.clone())
            .or_insert_with(|| VariantPlan {
                selector_regexes: Arc::new(compile_selector_regexes_with_html_ids(
                    &key.selector_attributes,
                    &key.component_selector_attributes,
                    key.html_ids,
                )),
                policies: Vec::new(),
            });
        if selection.demands_text_imports {
            merge_sorted(&mut variant.policies, [selection.policy]);
        }
    }

    pub(crate) fn variants(
        &self,
    ) -> impl Iterator<Item = (&PlaywrightOccurrenceKey, &VariantPlan)> {
        self.variants.iter()
    }
}

impl PlaywrightOccurrenceKey {
    pub(crate) fn new(
        navigation_helpers: &[String],
        selector_wrappers: &[crate::config::v2::schema::PlaywrightSelectorWrapper],
        selector_attributes: &[String],
        component_selector_attributes: &BTreeMap<String, String>,
        html_ids: bool,
        test_id_attributes: &[String],
    ) -> Self {
        Self {
            navigation_helpers: sorted(navigation_helpers),
            selector_wrappers: sorted(selector_wrappers),
            selector_attributes: sorted(selector_attributes),
            component_selector_attributes: component_selector_attributes.clone(),
            html_ids,
            test_id_attributes: sorted(test_id_attributes),
        }
    }
}

fn sorted<T: Ord + Clone>(values: &[T]) -> Vec<T> {
    values
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
