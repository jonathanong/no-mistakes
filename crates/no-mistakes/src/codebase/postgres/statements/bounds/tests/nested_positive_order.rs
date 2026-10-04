use super::facts;

#[test]
fn nested_positive_order_caps_remain_distinct_from_implicit_groups() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/nested-positive-order.sql"
    ));
    assert_eq!(
        facts(sql)
            .iter()
            .map(|fact| fact.query.capped)
            .collect::<Vec<_>>(),
        [true, true, true, false, false, false, false, false]
    );
}
