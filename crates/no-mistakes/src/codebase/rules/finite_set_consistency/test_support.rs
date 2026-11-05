use super::yaml::{extract_yaml_sequence_value, extract_yaml_string_selector_value};
use std::collections::BTreeSet;

pub(super) fn extract_yaml_sequence(source: &str, key: &str) -> BTreeSet<String> {
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(source) else {
        return BTreeSet::new();
    };
    extract_yaml_sequence_value(&value, key)
}

pub(super) fn extract_yaml_string_selector(source: &str, selector: &str) -> BTreeSet<String> {
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(source) else {
        return BTreeSet::new();
    };
    extract_yaml_string_selector_value(&value, selector)
}
