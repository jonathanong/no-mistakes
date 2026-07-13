use super::{BTreeMap, BTreeSet, PlaywrightFileFactPlan, PlaywrightOccurrenceKey};

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
