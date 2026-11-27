use super::shape;

#[test]
fn each_table_arm_keeps_its_quoted_temporary_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/multi-table-arms.sql"
    ));
    assert_eq!(shape(sql), ["select: ((opaque) (opaque)) (accounts)"]);
}
