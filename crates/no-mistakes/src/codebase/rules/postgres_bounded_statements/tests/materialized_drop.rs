#[test]
fn materialized_view_cascade_restores_catalog_findings() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-materialized-drop.sql"));
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
            ("orders", 11),
            ("order_lines", 12),
            ("orders", 19),
            ("order_lines", 26),
            ("orders", 30),
            ("orders", 36)
        ]
        .map(|(table, line)| (table.to_string(), line))
    );
}
