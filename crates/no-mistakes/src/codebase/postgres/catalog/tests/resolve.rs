use super::super::snapshot::{Snapshot, SnapshotIndex, SnapshotIndexKey, SnapshotTable};
use super::super::{
    ArbiterTable, CanonicalIndex, CanonicalOrderKey, ResolvedArbiter, SchemaCatalog,
};
use std::collections::BTreeMap;

fn key(expression: &str) -> CanonicalOrderKey {
    CanonicalOrderKey {
        expression: expression.to_owned(),
        ascending: true,
        nulls_first: false,
    }
}

fn index(name: &str, keys: &[&str], constraint_backed: bool) -> CanonicalIndex {
    CanonicalIndex {
        name: name.to_owned(),
        constraint_backed,
        keys: keys.iter().map(|expression| key(expression)).collect(),
        predicate: None,
    }
}

fn catalog() -> SchemaCatalog {
    let mut tables = BTreeMap::new();
    let mut unique_constraints = BTreeMap::new();
    unique_constraints.insert("user_email_key".to_owned(), vec!["email".to_owned()]);
    tables.insert(
        "users".to_owned(),
        ArbiterTable {
            indexes: vec![
                index("users_id_key", &["id"], true),
                index("users_email_index", &["email"], true),
                index("users_pair_first", &["first", "second"], false),
                index("users_pair_second", &["second", "first"], false),
                CanonicalIndex {
                    predicate: Some("is_live".to_owned()),
                    ..index("users_live_email", &["email"], false)
                },
            ],
            unique_constraints,
        },
    );
    tables.insert(
        "partial_users".to_owned(),
        ArbiterTable {
            indexes: vec![CanonicalIndex {
                predicate: Some("is_live".to_owned()),
                ..index("partial_users_email", &["email"], false)
            }],
            unique_constraints: BTreeMap::new(),
        },
    );
    SchemaCatalog {
        tables,
        ..Default::default()
    }
}

#[test]
fn resolver_distinguishes_exact_ambiguous_and_unresolved_catalog_arbiters() {
    let catalog = catalog();
    assert!(matches!(
        catalog.resolve_columns("users", &["id".to_owned()], None),
        ResolvedArbiter::Exact(_)
    ));
    assert_eq!(
        catalog.resolve_columns("users", &["first".to_owned(), "second".to_owned()], None,),
        ResolvedArbiter::Ambiguous
    );
    assert_eq!(
        catalog.resolve_columns("users", &["missing".to_owned()], None),
        ResolvedArbiter::Unresolved
    );
    assert!(matches!(
        catalog.resolve_columns("users", &["email".to_owned()], Some("is_live")),
        ResolvedArbiter::Exact(_)
    ));
}

#[test]
fn constraint_resolution_falls_back_to_a_differently_named_backing_index() {
    let catalog = catalog();
    assert!(matches!(
        catalog.resolve_constraint("users", "user_email_key"),
        ResolvedArbiter::Exact(index) if index.name == "users_email_index"
    ));
    assert_eq!(
        catalog.resolve_constraint("users", "missing_constraint"),
        ResolvedArbiter::Unresolved
    );
    assert_eq!(
        catalog.resolve_constraint("missing_table", "user_email_key"),
        ResolvedArbiter::Unresolved
    );
}

#[test]
fn catalog_prefixes_ignore_qualifiers_but_exclude_partial_indexes() {
    let catalog = catalog();
    assert!(catalog.has_canonical_prefix("users", &[key("input.id")]));
    assert!(!catalog.has_canonical_prefix("partial_users", &[key("email")]));
    assert!(!catalog.has_canonical_prefix("missing_table", &[key("id")]));
}

#[test]
fn schema_qualified_tables_keep_distinct_catalog_identities() {
    let index = |expression: &str| SnapshotIndex {
        access_method: "btree".to_owned(),
        unique: true,
        valid: true,
        ready: true,
        keys: vec![SnapshotIndexKey {
            expression: expression.to_owned(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let table = |index_name: &str, expression: &str| SnapshotTable {
        indexes: BTreeMap::from([(index_name.to_owned(), index(expression))]),
        ..Default::default()
    };
    let catalog = SchemaCatalog::from_snapshot(
        "schema.json",
        Snapshot {
            format_version: 2,
            tables: BTreeMap::from([
                ("public.items".to_owned(), table("public_items_key", "id")),
                (
                    "archive.items".to_owned(),
                    table("archive_items_key", "archive_id"),
                ),
            ]),
            ..Default::default()
        },
    )
    .unwrap();

    assert!(catalog.has_canonical_prefix("public.items", &[key("id")]));
    assert!(!catalog.has_canonical_prefix("archive.items", &[key("id")]));
    assert!(catalog.has_canonical_prefix("archive.items", &[key("archive_id")]));
    assert!(!catalog.has_canonical_prefix("items", &[key("id")]));
}

#[test]
fn arbiter_indexes_keep_ready_btree_expression_keys() {
    let key = |expression: &str, descending: bool, nulls_first: bool| SnapshotIndexKey {
        expression: expression.to_owned(),
        descending,
        nulls_first,
        ..Default::default()
    };
    let index = |unique: bool, primary: bool, valid: bool, ready: bool, keys| SnapshotIndex {
        access_method: "btree".to_owned(),
        unique,
        primary,
        valid,
        ready,
        keys,
        predicate: Some("is_live".to_owned()),
        ..Default::default()
    };
    let catalog = SchemaCatalog::from_snapshot(
        "schema.json",
        Snapshot {
            format_version: 2,
            tables: BTreeMap::from([(
                "widgets".to_owned(),
                SnapshotTable {
                    indexes: BTreeMap::from([
                        (
                            "invalid".to_owned(),
                            index(true, false, false, true, vec![key("id", false, false)]),
                        ),
                        (
                            "not_ready".to_owned(),
                            index(true, false, true, false, vec![key("id", false, false)]),
                        ),
                        ("empty".to_owned(), index(true, false, true, true, vec![])),
                        (
                            "blank_expr".to_owned(),
                            index(true, false, true, true, vec![key("", false, false)]),
                        ),
                        (
                            "primary_only".to_owned(),
                            index(false, true, true, true, vec![key("pk", false, false)]),
                        ),
                        (
                            "kept".to_owned(),
                            index(
                                true,
                                false,
                                true,
                                true,
                                vec![key("", true, true), key("id", true, true)],
                            ),
                        ),
                    ]),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        },
    )
    .unwrap();

    assert!(matches!(
        catalog.resolve_columns("widgets", &["pk".to_owned()], Some("is_live")),
        ResolvedArbiter::Exact(index) if index.name == "primary_only"
    ));
    match catalog.resolve_columns("widgets", &["id".to_owned()], Some("is_live")) {
        ResolvedArbiter::Exact(index) => {
            assert_eq!(index.name, "kept");
            assert_eq!(index.predicate.as_deref(), Some("is_live"));
            assert_eq!(index.keys.len(), 1);
            assert!(!index.keys[0].ascending);
            assert!(index.keys[0].nulls_first);
        }
        other => panic!("expected the kept index, got {other:?}"),
    }
}
