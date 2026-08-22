use super::*;
use std::collections::BTreeMap;
use std::path::Path;

#[test]
fn test_extract_app_selectors_basic() {
    let path = Path::new("test.tsx");
    let source = r#"
        export function Page() {
            return (
                <>
                    <button data-testid="save-btn" />
                    <CustomButton customTestId="custom-btn" />
                    <div ignored-attr="ignored" />
                </>
            );
        }
    "#;

    let attributes = vec!["data-testid".to_string()];
    let mut component_attributes = BTreeMap::new();
    component_attributes.insert("customTestId".to_string(), "data-testid".to_string());

    let selectors =
        extract_app_selectors(path, source, &attributes, &component_attributes).unwrap();

    let mut values: Vec<_> = selectors
        .iter()
        .map(|s| (s.attribute.clone(), s.display_value()))
        .collect();
    values.sort();

    assert_eq!(
        values,
        vec![
            ("data-testid".to_string(), "custom-btn".to_string()),
            ("data-testid".to_string(), "save-btn".to_string()),
        ]
    );
}

#[test]
fn test_extract_app_selectors_empty() {
    let path = Path::new("test.tsx");
    let source = "export const x = 1;";

    let attributes = vec!["data-testid".to_string()];
    let component_attributes = BTreeMap::new();

    let selectors =
        extract_app_selectors(path, source, &attributes, &component_attributes).unwrap();
    assert!(selectors.is_empty());
}

#[test]
fn collect_app_selectors_with_sources_reuses_the_prepared_store() {
    use crate::codebase::ts_source::{FileInventory, SourceStore};
    use std::sync::Arc;

    let root = crate::playwright::test_support::fixture_path(&[
        "ast-snippets",
        "selectors",
        "collect-app",
    ]);
    let page = root.join("page.tsx");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&page)));
    let store = SourceStore::new(inventory);
    let attributes = vec!["data-testid".to_string()];

    let first = collect_app_selectors_with_sources(&root, &attributes, Some(&store)).unwrap();
    let reads = store.physical_read_count();
    assert_eq!(first.len(), 1);
    assert!(reads >= 1);

    let second = collect_app_selectors_with_sources(&root, &attributes, Some(&store)).unwrap();
    assert_eq!(second.len(), 1);
    assert_eq!(store.physical_read_count(), reads);
}

#[test]
fn collect_app_selectors_source_does_not_read_the_filesystem_directly() {
    let source = include_str!("../extract_app.rs");
    assert!(!source.contains("std::fs::read_to_string"));
    assert!(source.contains("read_prepared_or_open"));
}
