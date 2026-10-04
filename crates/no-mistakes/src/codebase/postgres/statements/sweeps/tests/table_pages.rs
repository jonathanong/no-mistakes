#[test]
fn standalone_table_pages_reuse_single_relation_sweep_facts() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/review-followups/sql/table-pages.sql"));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(
        !facts.parse_failed,
        "{:?}",
        crate::codebase::postgres::parse_postgres_sql(sql)
    );
    assert_eq!(
        facts
            .sweeps
            .iter()
            .map(|sweep| (
                sweep.table.as_str(),
                sweep.line,
                sweep.order_columns.as_slice()
            ))
            .collect::<Vec<_>>(),
        [
            ("orders", 2, &["id".to_owned()][..]),
            ("public.orders", 3, &["id".to_owned()][..]),
            ("Order Items", 4, &["id".to_owned()][..]),
            ("orders", 5, &["id".to_owned()][..]),
            ("orders", 10, &["id".to_owned()][..]),
            ("orders", 14, &["id".to_owned()][..]),
        ]
    );
}
