#[test]
fn nested_positive_order_caps_do_not_hide_uncapped_siblings() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/nested-positive-order.sql"
    ));
    assert!(crate::codebase::postgres::parse_postgres_sql(sql).is_ok());
    assert_eq!(
        super::unbounded(sql),
        [
            ("orders".into(), 5),
            ("orders".into(), 6),
            ("orders".into(), 7),
            ("accounts".into(), 8),
            ("orders".into(), 9),
        ]
    );
}
