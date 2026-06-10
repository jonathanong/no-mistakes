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
