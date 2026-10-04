#[test]
fn physical_schema_renames_restore_catalog_findings() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-schema-rename.sql"));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let catalog = super::catalog();
    let found = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect::<Vec<_>>();
    assert_eq!(
        found,
        [
            ("orders", 8),
            ("order_lines", 9),
            ("orders", 16),
            ("orders", 23),
            ("order_lines", 31),
            ("orders", 38),
            ("orders", 43),
            ("orders", 50),
            ("orders", 58)
        ]
        .map(|(table, line)| (table.to_string(), line))
    );
}
