use super::names;

#[test]
fn prepared_select_into_shadows_only_after_execute() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-prepared-into.sql"
    ));
    assert_eq!(names(sql), ["accounts", "accounts", "accounts", "accounts"]);
}

#[test]
fn rejected_prepared_transitions_keep_catalog_reads_visible() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-prepared-validation.sql"
    ));
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        27
    );
    assert_eq!(names(sql), ["accounts", "accounts", "accounts", "accounts"]);
}
