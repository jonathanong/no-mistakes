use super::shape;

#[test]
fn materialized_drops_preserve_temporary_wrong_kind_namesakes() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-materialized-namesake.sql"));
    assert_eq!(
        shape(sql),
        [
            "select: opaque",
            "select: opaque",
            "select: order_lines",
            "select: opaque",
            "select: opaque",
            "select: orders"
        ]
    );
}
