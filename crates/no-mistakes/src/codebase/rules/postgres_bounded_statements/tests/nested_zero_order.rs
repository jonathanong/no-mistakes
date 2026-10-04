use super::{catalog, extract_sql_statement_facts, offenders};

#[test]
fn nested_zero_caps_survive_outer_order_expansion() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/nested-zero-order.sql"
    ));
    let parsed = crate::codebase::postgres::parse_postgres_sql(sql).unwrap();
    assert_eq!(parsed.len(), 8);
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 8);
    let catalog = catalog();
    let findings: Vec<_> = facts
        .bounds
        .iter()
        .map(|fact| offenders(fact, &catalog))
        .collect();
    assert_eq!(
        findings.iter().map(Vec::len).collect::<Vec<_>>(),
        [0, 0, 0, 0, 0, 1, 1, 1]
    );
    assert_eq!(findings[5][0].table, "orders");
    assert_eq!(findings[5][0].line, 8);
}
