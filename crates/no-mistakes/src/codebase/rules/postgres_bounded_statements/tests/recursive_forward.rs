use super::names;

#[test]
fn recursive_forward_ctes_report_the_unbounded_catalog_leaf() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/recursive-forward-bounds.sql"
    ));
    assert_eq!(names(sql), ["accounts", "accounts"]);
}
