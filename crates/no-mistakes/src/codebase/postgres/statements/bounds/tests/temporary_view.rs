use super::shape;

#[test]
fn temporary_view_dependencies_include_scalar_subqueries_without_cte_aliases() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-view-all-dependencies.sql"
    ));
    assert_eq!(
        shape(sql),
        [
            "select: opaque",
            "select: accounts",
            "select: orders",
            "select: accounts"
        ]
    );
}

#[test]
fn temporary_view_dependencies_respect_cte_declaration_visibility() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-view-cte-visibility.sql"
    ));
    assert_eq!(
        shape(sql),
        [
            "select: opaque",
            "select: orders",
            "select: accounts",
            "select: orders",
            "select: opaque",
            "select: accounts",
            "select: accounts"
        ]
    );
}

#[test]
fn table_function_name_does_not_make_view_temporary() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-view-table-function.sql"
    ));
    assert_eq!(shape(sql), ["select: accounts", "select: accounts"]);
}

#[test]
fn definite_temp_read_survives_ambiguous_namesake_in_same_view() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-mixed-view-dependencies.sql"
    ));
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        6
    );
    assert_eq!(
        shape(sql),
        ["select: opaque", "select: opaque public.accounts[id=#0]"]
    );
}

#[test]
fn mixed_view_dependencies_follow_definite_rename_and_cascade() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-mixed-view-rename.sql"
    ));
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        8
    );
    assert_eq!(shape(sql), ["select: opaque", "select: orders"]);
}
