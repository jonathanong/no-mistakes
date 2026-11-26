use super::extract_sql_statement_facts;

#[test]
fn select_all_keeps_row_sweeps_while_deduplicating_projections_do_not() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/select-all/sql/pages.sql"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(
        facts
            .sweeps
            .iter()
            .map(|sweep| sweep.line)
            .collect::<Vec<_>>(),
        [2, 3, 6, 7]
    );
    assert!(facts
        .sweeps
        .iter()
        .all(|sweep| sweep.order_columns == ["account_id"]));
    assert_eq!(facts.sweeps[0].conjuncts, facts.sweeps[1].conjuncts);
}
