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
}
