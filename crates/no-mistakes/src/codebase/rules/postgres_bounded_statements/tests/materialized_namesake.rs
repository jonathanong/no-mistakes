#[test]
fn materialized_drops_preserve_temporary_wrong_kind_namesakes() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-materialized-namesake.sql"));
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
        [("order_lines", 9), ("orders", 17)].map(|(table, line)| (table.to_string(), line))
    );
}
