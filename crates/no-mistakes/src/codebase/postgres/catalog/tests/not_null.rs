use super::super::SchemaCatalog;
use std::path::PathBuf;

#[test]
fn issue_3462_shared_bare_name_does_not_prove_not_null() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/issue-3462/schema.json");
    let catalog = SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        catalog
            .relation("archive.budgets")
            .map(|table| table.name.as_str()),
        Some("budgets")
    );
    assert!(catalog.bounded_relation("archive.budgets").is_none());
    assert!(catalog.bounded_relation("budgets").is_none());
    assert!(!catalog.column_is_not_null("archive.budgets", "cap"));
    assert!(!catalog.column_is_not_null("budgets", "cap"));
    assert!(catalog.bounded_relation("safe").is_some());
    assert!(catalog.column_is_not_null("safe", "cap"));
    assert!(catalog.column_is_not_null("safe", "actor_id"));
    assert!(!catalog.column_is_not_null("safe", "missing"));
}
