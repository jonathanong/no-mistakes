use crate::playwright::selectors::regex_mod::{
    compile_selector_regexes, compile_selector_regexes_with_html_ids,
};
use crate::playwright::selectors::HTML_ID_ATTRIBUTE;
use std::collections::BTreeMap;

#[test]
fn test_compile_selector_regexes() {
    let attributes = vec!["data-testid".to_string(), "data-cy".to_string()];
    let mut component_attributes = BTreeMap::new();
    component_attributes.insert("Button".to_string(), "data-test-id".to_string());

    let regexes = compile_selector_regexes(&attributes, &component_attributes);

    assert_eq!(regexes.app_attributes, attributes);
    assert_eq!(regexes.component_attributes, component_attributes);
    assert!(!regexes.html_ids);

    let mut expected_playwright_attributes = vec![
        "data-cy".to_string(),
        "data-test-id".to_string(),
        "data-testid".to_string(),
    ];
    expected_playwright_attributes.sort();

    let actual_playwright_attributes: Vec<_> = regexes
        .playwright_attributes
        .iter()
        .map(|ar| ar.attribute.clone())
        .collect();

    assert_eq!(actual_playwright_attributes, expected_playwright_attributes);

    // Verify dedup works
    let mut component_attributes_with_dup = BTreeMap::new();
    component_attributes_with_dup.insert("Button".to_string(), "data-testid".to_string());

    let regexes_dup = compile_selector_regexes(&attributes, &component_attributes_with_dup);
    let mut expected_dup = vec!["data-cy".to_string(), "data-testid".to_string()];
    expected_dup.sort();
    let actual_dup: Vec<_> = regexes_dup
        .playwright_attributes
        .iter()
        .map(|ar| ar.attribute.clone())
        .collect();
    assert_eq!(actual_dup, expected_dup);
}

#[test]
fn test_compile_selector_regexes_with_html_ids() {
    let attributes = vec!["data-testid".to_string()];
    let mut component_attributes = BTreeMap::new();
    component_attributes.insert("Button".to_string(), "data-test-id".to_string());

    let regexes = compile_selector_regexes_with_html_ids(&attributes, &component_attributes, true);

    assert!(regexes.html_ids);

    let mut expected_playwright_attributes = vec![
        "data-testid".to_string(),
        "data-test-id".to_string(),
        HTML_ID_ATTRIBUTE.to_string(),
    ];
    expected_playwright_attributes.sort();

    let actual_playwright_attributes: Vec<_> = regexes
        .playwright_attributes
        .iter()
        .map(|ar| ar.attribute.clone())
        .collect();

    assert_eq!(actual_playwright_attributes, expected_playwright_attributes);
}
