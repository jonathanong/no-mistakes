use super::extract_migration_facts;

#[test]
fn classifies_modifiers_and_alter_object_kinds() {
    let sql = include_str!("../../../../../fixtures/rules/postgres-statement-kinds/variants.sql");
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

#[test]
fn classifies_transaction_control_kinds_and_skips_procedural_delimiters() {
    let sql = include_str!(
        "../../../../../fixtures/rules/postgres-statement-kinds/transaction-control.sql"
    );
    let schema = extract_migration_facts(sql);
    let direct = crate::statements::extract_sql_statement_facts(sql);
    let expected = vec![
        (3, "BEGIN"),
        (4, "BEGIN"),
        (5, "START TRANSACTION"),
        (6, "START TRANSACTION"),
        (7, "SAVEPOINT"),
        (8, "ROLLBACK TO SAVEPOINT"),
        (9, "ROLLBACK TO SAVEPOINT"),
        (10, "RELEASE SAVEPOINT"),
        (11, "RELEASE SAVEPOINT"),
        (12, "COMMIT"),
        (13, "COMMIT"),
        (14, "END"),
        (15, "END"),
        (16, "ROLLBACK"),
        (17, "ROLLBACK"),
        (31, "COMMIT"),
        (32, "ROLLBACK"),
        (37, "COMMIT"),
        (38, "SAVEPOINT"),
        (40, "RELEASE SAVEPOINT"),
        (42, "ROLLBACK TO SAVEPOINT"),
        (46, "COMMIT"),
        (47, "ROLLBACK"),
        (52, "COMMIT"),
        (53, "ROLLBACK"),
        (57, "COMMIT"),
        (59, "COMMIT"),
        (61, "ROLLBACK"),
    ];
    for (label, kinds) in [
        ("schema", &schema.statement_kinds),
        ("direct", &direct.statement_kinds),
    ] {
        let mut actual: Vec<_> = kinds
            .iter()
            .filter(|item| {
                crate::migration::TRANSACTION_CONTROL_KINDS.contains(&item.kind.as_str())
            })
            .map(|item| (item.line, item.kind.as_str()))
            .collect();
        actual.sort_unstable();
        assert_eq!(actual, expected, "{label}: {kinds:?}");
    }
}
