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

#[test]
fn skip_dynamic_recovery_keeps_literals_and_prop_defaults_only() {
    let path = Path::new("page.tsx");
    let source = r#"
        export function Link({ 'data-pw': dataPw = 'rss-feed-link' }) {
            return <a data-pw={dataPw}>RSS</a>;
        }
        export function Page() {
            const ID = 'from-const';
            return (
                <>
                    <button data-testid="save" />
                    <button data-testid={"publish"} />
                    <button data-testid={ID} />
                    <button data-testid={true ? 'a' : 'b'} />
                </>
            );
        }
    "#;
    let regexes = crate::playwright::selectors::compile_selector_regexes(
        &["data-testid".to_string(), "data-pw".to_string()],
        &BTreeMap::new(),
    );
    let visible: crate::fx::PathSet = [path.to_path_buf()].into_iter().collect();
    let skipped = crate::ast::with_program(path, source, |program, source| {
        extract_app_selectors_from_program_from_visible_deferred(
            path, source, program, &regexes, &visible, false,
        )
    })
    .unwrap();
    let recovered = crate::ast::with_program(path, source, |program, source| {
        extract_app_selectors_from_program_from_visible_deferred(
            path, source, program, &regexes, &visible, true,
        )
    })
    .unwrap();

    let skipped_values: Vec<_> = skipped
        .iter()
        .map(|selector| {
            (
                selector.display_value(),
                selector.is_literal_exact(),
                selector.unsupported_dynamic(),
            )
        })
        .collect();
    assert!(skipped_values.contains(&("save".to_string(), true, false)));
    assert!(skipped_values.contains(&("publish".to_string(), true, false)));
    assert!(skipped_values.contains(&("rss-feed-link".to_string(), true, false)));
    assert!(skipped
        .iter()
        .any(|selector| { selector.display_value() == "{ID}" && selector.unsupported_dynamic() }));
    assert!(skipped.iter().any(|selector| {
        selector.unsupported_dynamic() && selector.display_value().contains('?')
    }));
    assert!(recovered
        .iter()
        .any(|selector| { selector.display_value() == "from-const" && selector.recovered }));
}
