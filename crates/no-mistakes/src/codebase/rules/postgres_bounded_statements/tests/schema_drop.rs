#[test]
fn physical_schema_cascades_restore_findings_and_rollback() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-schema-drop.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let catalog = super::catalog();
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    assert_eq!(
        found,
        [
            ("orders", 11),
            ("order_lines", 12),
            ("orders", 19),
            ("order_lines", 20),
            ("orders", 25),
            ("orders", 28)
        ]
        .map(|(table, line)| (table.to_string(), line))
    );
}
