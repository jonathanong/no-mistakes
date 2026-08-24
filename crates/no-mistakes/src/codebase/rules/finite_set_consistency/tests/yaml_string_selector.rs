use super::super::extract::{
    extract_sql_enum, extract_ts_const_object_keys, extract_ts_const_object_property,
    extract_ts_string_union, extract_yaml_string_selector,
};
use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn extracts_terminal_strings_and_ignores_invalid_paths() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency/yaml-string-selector"),
    );
    let source = std::fs::read_to_string(root.join("selectors.yml")).unwrap();

    assert_eq!(
        extract_yaml_string_selector(&source, "rules.[].options.permanentPackages.[].name"),
        BTreeSet::from(["@acme/api".to_string(), "@acme/web".to_string()]),
    );
    assert_eq!(
        extract_yaml_string_selector(&source, "updates.0.name"),
        BTreeSet::from(["first".to_string()]),
    );
    assert_eq!(
        extract_yaml_string_selector(&source, "literal.[\"dotted.key\"]"),
        BTreeSet::from(["dotted".to_string()]),
    );
    assert_eq!(
        extract_yaml_string_selector(&source, "literal.[\"[]\"]"),
        BTreeSet::from(["brackets".to_string()]),
    );
    assert_eq!(
        extract_yaml_string_selector(&source, "literal.[\"0\"]"),
        BTreeSet::from(["zero".to_string()]),
    );
    assert!(extract_yaml_string_selector(&source, "rules.name").is_empty());
    assert!(
        extract_yaml_string_selector(&source, "rules.1.options.permanentPackages.[]").is_empty()
    );
    assert!(extract_yaml_string_selector(&source, "rules.[].options.mode").is_empty());
    for invalid in [
        "",
        "literal..key",
        "literal.[unterminated]",
        "literal.[\"unterminated]",
    ] {
        assert!(extract_yaml_string_selector(&source, invalid).is_empty());
    }
}

#[test]
fn extraction_helpers_return_empty_sets_when_targets_are_missing() {
    assert!(extract_ts_string_union("type Other = 'a';", "Missing").is_empty());
    assert!(extract_ts_const_object_keys("const Other = { a: 1 };", "Missing").is_empty());
    assert!(extract_ts_const_object_property(
        "const Other = { a: { slug: 'a' } };",
        "Missing",
        "slug"
    )
    .is_empty());
    assert!(extract_sql_enum("CREATE TYPE other AS ENUM ('a')", "missing").is_empty());
    assert!(extract_ts_const_object_keys(
        "const ROUTE_META: Record<string, string>;",
        "ROUTE_META"
    )
    .is_empty());
}

#[test]
fn extracts_const_object_keys_and_property_values() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/finite-set-consistency/fixture"),
    );
    let source = std::fs::read_to_string(root.join("src/types.ts")).unwrap();

    assert!(extract_ts_const_object_keys(&source, "ROUTE_META").contains("users"));
    assert!(extract_ts_const_object_property(&source, "ROUTE_META", "slug").contains("billing"));
}
