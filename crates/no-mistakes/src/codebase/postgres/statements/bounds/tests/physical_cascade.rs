use super::shape;

#[test]
fn temporary_view_dependencies_follow_physical_cascade() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-physical-cascade.sql"));
    assert_eq!(
        shape(sql),
        [
            "select: opaque",
            "select: opaque",
            "select: opaque",
            "select: orders",
            "select: order_lines",
            "select: orders",
            "select: opaque",
            "select: orders",
            "select: orders",
            "select: orders",
            "select: orders",
            "select: opaque",
            "select: order_lines",
            "select: orders",
            "select: orders"
        ]
    );
}
