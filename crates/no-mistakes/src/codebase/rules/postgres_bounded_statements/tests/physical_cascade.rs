#[test]
fn temporary_view_dependencies_follow_physical_cascade() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-physical-cascade.sql"));
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
            ("orders", 10),
            ("order_lines", 11),
            ("orders", 14),
            ("orders", 22),
            ("orders", 27),
            ("orders", 31),
            ("orders", 36),
            ("order_lines", 43),
            ("orders", 47),
            ("orders", 50)
        ]
        .map(|(table, line)| (table.to_string(), line))
    );
}
