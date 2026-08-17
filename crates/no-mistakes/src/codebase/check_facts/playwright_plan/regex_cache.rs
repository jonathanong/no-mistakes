use crate::playwright::selectors::{compile_selector_regexes_with_html_ids, SelectorRegexes};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub(super) struct SelectorRegexCache {
    entries: BTreeMap<SelectorRegexKey, Arc<SelectorRegexes>>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
struct SelectorRegexKey {
    selector_attributes: Vec<String>,
    component_selector_attributes: Vec<(String, String)>,
    html_ids: bool,
}

impl SelectorRegexCache {
    pub(super) fn get_or_compile(
        &mut self,
        selector_attributes: &[String],
        component_selector_attributes: &BTreeMap<String, String>,
        html_ids: bool,
    ) -> Arc<SelectorRegexes> {
        let key =
            SelectorRegexKey::new(selector_attributes, component_selector_attributes, html_ids);
        self.entries
            .entry(key)
            .or_insert_with(|| {
                Arc::new(compile_selector_regexes_with_html_ids(
                    selector_attributes,
                    component_selector_attributes,
                    html_ids,
                ))
            })
            .clone()
    }

    pub(super) fn extend(&mut self, other: Self) {
        for (key, regexes) in other.entries {
            self.entries.entry(key).or_insert(regexes);
        }
    }
}

impl SelectorRegexKey {
    fn new(
        selector_attributes: &[String],
        component_selector_attributes: &BTreeMap<String, String>,
        html_ids: bool,
    ) -> Self {
        Self {
            selector_attributes: selector_attributes.to_vec(),
            component_selector_attributes: component_selector_attributes
                .iter()
                .map(|(component, attribute)| (component.clone(), attribute.clone()))
                .collect(),
            html_ids,
        }
    }
}
