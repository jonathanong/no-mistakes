use super::super::SchemaCatalog;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres/catalog-logical-scope")
}

#[test]
fn logical_tables_keep_nested_parents_without_hiding_public_leaf_facts() {
    let json = std::fs::read_to_string(fixture_root().join("with-local-leaf/schema.json")).unwrap();
    let catalog = SchemaCatalog::from_json(&json).unwrap();
    assert!(catalog.tables().any(|table| table.name == "BadLeaf"));
    let logical = catalog
        .logical_tables()
        .map(|table| table.name.as_str())
        .collect::<Vec<_>>();
    assert!(!logical.contains(&"BadLeaf"));
    assert!(logical.contains(&"accounts"));
    assert!(logical.contains(&"parent_accounts"));
    assert!(logical.contains(&"nested_events_2026"));
}

#[test]
fn leaf_local_keys_do_not_change_other_configured_catalog_findings() {
    let run = |name: &str| {
        let fixture = crate::test_support::materialize_saved_fixture(&fixture_root().join(name));
        let config = fixture.path().join(".no-mistakes.yml");
        crate::codebase::rules::run_filesystem_rules(fixture.path(), Some(&config)).unwrap()
    };
    let baseline = run("baseline");
    let with_leaf = run("with-local-leaf");
    let unrelated = |findings: &[crate::codebase::rules::RuleFinding]| {
        findings
            .iter()
            .filter(|finding| finding.rule != "postgres-key-column-types")
            .cloned()
            .collect::<Vec<_>>()
    };
    assert!(!unrelated(&baseline).is_empty());
    assert_eq!(unrelated(&baseline), unrelated(&with_leaf));
    for rule in [
        "postgres-conflict-ordering",
        "postgres-lock-ordering",
        "postgres-duplicate-function-body",
        "postgres-explicit-columns",
        "postgres-bounded-statements",
        "postgres-key-column-types",
        "postgres-object-naming",
        "postgres-column-naming",
        "postgres-finite-text-columns",
        "postgres-array-columns",
        "postgres-column-requires-trigger",
        "postgres-required-comments",
        "postgres-table-shape",
        "postgres-status-with-lifecycle-timestamps",
        "postgres-required-predicates",
    ] {
        assert!(
            baseline.iter().any(|finding| finding.rule == rule),
            "missing exercised rule {rule}: {baseline:#?}"
        );
    }
    assert!(
        with_leaf
            .iter()
            .any(|finding| finding.rule == "postgres-key-column-types"
                && finding.message.contains("BadLeaf.BadLeaf_email_fkey")
                && finding.message.contains("text column email")),
        "{with_leaf:#?}"
    );
    assert!(baseline
        .iter()
        .all(|finding| !finding.message.contains("BadLeaf")));
}
