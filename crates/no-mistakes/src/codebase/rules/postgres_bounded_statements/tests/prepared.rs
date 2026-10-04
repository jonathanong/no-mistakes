use super::names;

#[test]
fn prepared_select_into_shadows_only_after_execute() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-prepared-into.sql"
    ));
    assert_eq!(names(sql), ["accounts", "accounts", "accounts"]);
}
