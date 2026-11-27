use super::shape;

#[test]
fn materialized_view_cascade_invalidates_temporary_dependents() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-materialized-drop.sql"));
    assert_eq!(
        shape(sql),
        [
            "select: opaque",
            "select: opaque",
            "select: opaque",
            "select: orders",
            "select: order_lines",
            "select: opaque",
            "select: orders",
            "select: order_lines",
            "select: opaque",
            "select: orders",
            "select: opaque",
            "select: orders"
        ]
    );
}
