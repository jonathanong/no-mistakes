use super::ChangesReport;
use std::collections::BTreeMap;

#[test]
fn preserves_only_usable_attribute_paths_in_changes_reports() {
    let fixture = crate::test_support::fixture_root("changes-attributes").join("report.json");
    let source = std::fs::read(fixture).unwrap();
    let report: ChangesReport = serde_json::from_slice(&source).unwrap();
    let fixtures = &report.categories["fixtures"];
    assert_eq!(fixtures.changed.len(), 6);
    assert_eq!(fixtures.added, ["added"]);
    assert_eq!(
        fixtures.attribute_paths,
        BTreeMap::from([(
            "portrait".to_owned(),
            vec!["/portrait/~1label~0".to_owned()]
        )])
    );
    let legacy = &report.categories["legacy"];
    assert_eq!(legacy.changed, ["existing"]);
    assert!(legacy.attribute_paths.is_empty());
}

#[test]
fn rejects_duplicate_and_malformed_attribute_paths_maps() {
    let root = crate::test_support::fixture_root("changes-attributes");
    let duplicate = std::fs::read(root.join("duplicate-attribute-paths.json")).unwrap();
    let error = serde_json::from_slice::<ChangesReport>(&duplicate)
        .unwrap_err()
        .to_string();
    assert!(error.contains("duplicate attributePaths key"), "{error}");

    let invalid = std::fs::read(root.join("invalid-attribute-paths.json")).unwrap();
    let error = serde_json::from_slice::<ChangesReport>(&invalid)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("a map of unique fixture IDs to JSON pointer lists"),
        "{error}"
    );
}
