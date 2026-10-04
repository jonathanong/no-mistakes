use super::names;

#[test]
fn scalar_view_dependencies_shadow_and_then_release_catalog_relations() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-view-all-dependencies.sql"
    ));
    assert_eq!(names(sql), ["accounts", "orders", "accounts"]);
}

#[test]
fn nonrecursive_cte_bodies_keep_temporary_view_dependencies() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-view-cte-visibility.sql"
    ));
    assert_eq!(
        names(sql),
        ["orders", "accounts", "orders", "accounts", "accounts"]
    );
}

#[test]
fn table_function_name_keeps_view_independent_of_temporary_table() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-view-table-function.sql"
    ));
    assert_eq!(names(sql), ["accounts", "accounts"]);
}

#[test]
fn scalar_table_view_dependencies_follow_exact_source_identities() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-view-scalar-table.sql"));
    assert_eq!(
        super::names(sql),
        ["orders", "orders", "orders", "orders", "orders"]
    );
}
