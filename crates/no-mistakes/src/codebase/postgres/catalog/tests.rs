use super::*;
#[test]
fn expressions_ignore_formatting_but_not_source_bindings() {
    assert!(expression_matches(
        "LOWER( category_text )",
        "lower(category_text)",
        false
    ));
    assert!(!expression_matches(
        "lower(input.category_text)",
        "lower(category_text)",
        false
    ));
    assert!(expression_matches(
        "lower(input.category_text)",
        "lower(category_text)",
        true
    ));
}
#[test]
fn expression_normalization_preserves_structural_parentheses() {
    assert_ne!(
        normalize_expression("(left_value + right_value) * multiplier"),
        normalize_expression("left_value + (right_value * multiplier)")
    );
}
#[test]
fn rejects_outside_catalog_paths() {
    let root = Path::new("/repo");
    assert!(catalog_path(root, "../schema.json").is_err());
    assert!(catalog_path(root, "/schema.json").is_err());
    assert_eq!(
        catalog_path(root, "db/schema.json").unwrap(),
        root.join("db/schema.json")
    );
}
