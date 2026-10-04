use super::names;

#[test]
fn permanent_intermediate_view_cascade_reveals_catalog_relations() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-permanent-view-chain.sql"));
    assert_eq!(names(sql), ["orders", "orders", "orders", "orders"]);
}

#[test]
fn physical_view_source_rename_reveals_temporary_dependent_after_cascade() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-permanent-view-rename.sql"));
    assert_eq!(names(sql), ["orders"]);
}

#[test]
fn schema_rename_moves_permanent_intermediary_identity_before_cascade() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-permanent-view-schema-rename.sql"));
    assert_eq!(names(sql), ["orders"]);
}

#[test]
fn ambiguous_source_rename_preserves_later_catalog_finding() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-permanent-view-ambiguous-rename.sql"));
    assert_eq!(names(sql), ["orders"]);
}

#[test]
fn permanent_view_node_rename_preserves_cascade_identity_and_failed_ddl() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-permanent-view-node-rename.sql"));
    assert_eq!(
        names(sql),
        ["orders", "orders", "orders", "orders", "orders", "orders"]
    );
}

#[test]
fn bare_view_collision_preserves_each_schema_candidate() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-permanent-view-bare-collision.sql"));
    assert_eq!(names(sql), ["orders", "orders", "orders"]);
}
