use super::shape;

#[test]
fn physical_schema_cascades_restore_findings_and_rollback() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-schema-drop.sql"
    ));
    assert_eq!(
        shape(sql),
        [
            "select: opaque",
            "select: opaque",
            "select: opaque",
            "select: orders",
            "select: order_lines",
            "select: opaque",
            "select: opaque",
            "select: orders",
            "select: order_lines",
            "select: opaque",
            "select: orders",
            "select: orders",
            "select: opaque",
            "select: opaque",
            "select: opaque"
        ]
    );
}
