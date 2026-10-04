use super::shape;

#[test]
fn a_table_arm_names_a_cte_when_one_has_that_name() {
    assert_eq!(
        shape("WITH c AS (SELECT * FROM orders) SELECT 1 UNION ALL TABLE c"),
        ["select: () ((orders))"]
    );
    // A schema-qualified name is the table, whatever the CTE is called.
    assert_eq!(
        shape("WITH c AS (SELECT * FROM orders) SELECT 1 UNION ALL TABLE public.c"),
        ["select: () (public.c)"]
    );
}

#[test]
fn quoted_table_arms_keep_their_source_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted.sql"
    ));
    let found = shape(sql);
    assert!(found.contains(&"select: () (accounts)".to_string()));
    assert!(found.contains(&"select: () (public.accounts)".to_string()));
    assert_eq!(
        found
            .iter()
            .filter(|shape| shape.contains("opaque"))
            .count(),
        6
    );
}

#[test]
fn pg_temp_into_and_create_track_bare_temporary_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-qualified.sql"
    ));
    assert_eq!(
        shape(sql),
        [
            "select: capped orders",
            "select: opaque",
            "select: opaque",
            "select: accounts",
            "select: opaque",
            "select: accounts",
            "select: capped orders",
            "select: accounts"
        ]
    );
}

#[test]
fn temporary_relation_identity_tracks_source_statement_order() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-relations.sql"
    ));
    let shapes = shape(sql);
    assert_eq!(shapes[0], "select: accounts");
    assert_eq!(shapes[1], "select: opaque");
    assert_eq!(shapes[2], "select: opaque");
    assert_eq!(shapes[3], "select: opaque");
    assert_eq!(shapes[4], "select: public.accounts");
    assert_eq!(shapes[6], "select: accounts");
}
