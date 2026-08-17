use super::SelectorRegexCache;
use std::collections::BTreeMap;
use std::sync::Arc;

#[test]
fn get_or_compile_reuses_the_same_regex_set_for_identical_attributes() {
    let mut cache = SelectorRegexCache::default();
    let attributes = vec!["data-pw".to_string()];
    let components = BTreeMap::from([("Button".to_string(), "data-testid".to_string())]);
    let first = cache.get_or_compile(&attributes, &components, true);
    let second = cache.get_or_compile(&attributes, &components, true);
    assert!(Arc::ptr_eq(&first, &second));
    let without_html_ids = cache.get_or_compile(&attributes, &components, false);
    assert!(!Arc::ptr_eq(&first, &without_html_ids));
}

#[test]
fn get_or_compile_reuses_reversed_and_duplicated_selector_attributes() {
    let mut cache = SelectorRegexCache::default();
    let components = BTreeMap::new();
    let first = cache.get_or_compile(
        &["data-b".into(), "data-a".into(), "data-b".into()],
        &components,
        false,
    );
    let second = cache.get_or_compile(&["data-a".into(), "data-b".into()], &components, false);
    assert!(Arc::ptr_eq(&first, &second));
}

#[test]
fn extend_keeps_the_first_compiled_entry_for_a_key() {
    let attributes = vec!["data-pw".to_string()];
    let components = BTreeMap::new();
    let mut left = SelectorRegexCache::default();
    let kept = left.get_or_compile(&attributes, &components, false);
    let mut right = SelectorRegexCache::default();
    let _other = right.get_or_compile(&attributes, &components, false);
    left.extend(right);
    let reused = left.get_or_compile(&attributes, &components, false);
    assert!(Arc::ptr_eq(&kept, &reused));
}
