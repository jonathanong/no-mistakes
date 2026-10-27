#[test]
fn proven_physical_shadow_drop_exposes_the_underlying_temporary_rows() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-shadow-drop.sql"
    ));
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        13
    );
}

#[test]
fn unknown_shadow_drop_never_lends_catalog_keys_to_the_join() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-shadow-drop-unknown.sql"
    ));
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        10
    );
}

#[test]
fn database_qualified_shadow_drop_requires_a_matching_current_database() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-shadow-drop-database.sql"
    ));
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        5
    );
}

#[test]
fn recreated_physical_shadows_never_retire_the_underlying_temp_relation() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture");
    for fixture in [
        "temporary-shadow-recreate-table.sql",
        "temporary-shadow-recreate-view.sql",
        "temporary-shadow-recreate-materialized.sql",
    ] {
        let sql = std::fs::read_to_string(root.join("sql").join(fixture)).unwrap();
        assert_eq!(
            crate::codebase::postgres::parse_postgres_sql(&sql)
                .unwrap()
                .len(),
            7
        );
    }
}
