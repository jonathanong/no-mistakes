use super::extract_migration_facts;

#[test]
fn classifies_modifiers_and_alter_object_kinds() {
    let sql =
        include_str!("../../../../../../../fixtures/rules/postgres-statement-kinds/variants.sql");
    let facts = extract_migration_facts(sql);
    let kinds: Vec<_> = facts
        .statement_kinds
        .iter()
        .map(|kind| kind.kind.as_str())
        .collect();
    for (kind, count) in [
        ("CREATE TABLE", 3),
        ("CREATE INDEX", 1),
        ("CREATE VIEW", 2),
        ("CREATE FUNCTION", 1),
        ("CREATE PROCEDURE", 1),
        ("CREATE TRIGGER", 1),
    ] {
        assert_eq!(
            kinds.iter().filter(|item| **item == kind).count(),
            count,
            "{kind}: {facts:?}"
        );
    }
    let alterations: Vec<_> = facts
        .statement_kinds
        .iter()
        .filter(|kind| kind.kind.starts_with("ALTER"))
        .map(|kind| (kind.line, kind.kind.as_str()))
        .collect();
    assert_eq!(
        alterations,
        [
            (10, "ALTER VIEW"),
            (11, "ALTER VIEW"),
            (12, "ALTER TRIGGER"),
            (13, "ALTER FUNCTION"),
            (14, "ALTER PROCEDURE"),
            (15, "ALTER INDEX"),
            (16, "ALTER TYPE")
        ]
    );
    // Incomplete ALTER statements and incompatible CREATE modifiers have no kind.
    assert!(facts.statement_kinds.iter().all(|kind| kind.line < 18));
}
