use super::super::RelationKind;
use super::load_fixture;

#[test]
fn sparse_snapshot_uses_field_defaults() {
    let catalog = load_fixture("sparse.json").unwrap();
    let table = catalog.table("bare").unwrap();
    assert_eq!(table.relation_kind, RelationKind::Table);
    assert!(table.columns.is_empty());
    assert_eq!(table.primary_key, None);
    assert_eq!(table.partition_key, None);
    assert!(table.triggers.is_empty());
    assert!(catalog.functions().next().is_none());
    assert!(catalog.enums().next().is_none());
    assert!(catalog.views().next().is_none());
}

#[test]
fn a_catalog_must_state_coverage_and_every_column_type() {
    // Only `no-mistakes postgres catalog` output is accepted: nothing is assumed complete.
    let hint = "generate the catalog with `no-mistakes postgres catalog`";
    let missing = load_fixture("missing-coverage.json").unwrap_err();
    assert!(
        missing.contains("has an invalid schema: missing field `coverage`"),
        "{missing}"
    );
    assert!(missing.contains(hint), "{missing}");
    let unknown = load_fixture("unknown-coverage.json").unwrap_err();
    assert!(unknown.contains("unknown variant `partial`"), "{unknown}");
    let untyped = load_fixture("missing-data-type.json").unwrap_err();
    assert!(
        untyped.contains("invalid schema: tables.accounts.columns.id: missing field `dataType`"),
        "{untyped}"
    );
    assert!(untyped.contains(hint), "{untyped}");
}

#[test]
fn another_producers_shapes_fail_with_the_failing_field() {
    let renamed = load_fixture("foreign-type-key.json").unwrap_err();
    assert!(
        renamed.contains("tables.accounts.columns.id: missing field `dataType`"),
        "{renamed}"
    );
    let trigger = load_fixture("foreign-trigger-string.json").unwrap_err();
    assert!(
        trigger.contains("tables.accounts.triggers.touch: invalid type: string"),
        "{trigger}"
    );
    assert!(
        trigger.contains("expected struct SnapshotTrigger"),
        "{trigger}"
    );
    let version = load_fixture("wrong-version.json").unwrap_err();
    assert!(
        version.contains("no-mistakes postgres catalog"),
        "{version}"
    );
}

#[test]
fn snapshot_without_tables_has_no_column_locations() {
    let catalog = load_fixture("no-tables.json").unwrap();
    assert!(catalog.tables().next().is_none());
    assert_eq!(catalog.column_line("missing", "column"), 1);
}

#[test]
fn snapshot_load_reports_unsupported_and_unreadable_fields() {
    let strategy = load_fixture("bad-strategy.json").unwrap_err();
    assert!(strategy.contains("table broken has unsupported partition strategy MAGIC"));
    let relation = load_fixture("bad-relation.json").unwrap_err();
    assert!(relation.contains("table broken has unsupported relation kind view"));
    let generated = load_fixture("bad-generated.json").unwrap_err();
    assert!(generated.contains("table broken column id has unsupported generated kind always"));
    let trigger = load_fixture("bad-trigger.json").unwrap_err();
    assert!(trigger.contains("table broken trigger bad has an unreadable definition"));
    let syntax = load_fixture("bad-partition-syntax.json").unwrap_err();
    assert!(syntax.contains("table broken has unreadable partition key"));
    let extension = load_fixture("invalid-trailing-comma.json").unwrap_err();
    assert!(extension.contains("not valid JSONC"), "{extension}");
}
