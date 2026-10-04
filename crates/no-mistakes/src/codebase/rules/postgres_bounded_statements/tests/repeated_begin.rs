use super::names;

#[test]
fn repeated_begin_preserves_the_original_transaction_and_savepoints() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-repeated-begin.sql"
    ));
    assert_eq!(names(sql), ["accounts", "accounts", "orders"]);
}
