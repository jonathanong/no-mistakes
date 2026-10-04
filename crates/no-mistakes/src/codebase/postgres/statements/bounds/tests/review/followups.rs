use super::shape;

#[test]
fn stored_arrays_do_not_inherit_their_rows_bound() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/stored-array.sql"
    ));
    assert_eq!(
        shape(sql),
        [
            "update: accounts[id=stored-array#[1]] orders[id=value]",
            "update: accounts[id=stored-array#[1]] orders[id=value]",
            "delete: accounts[id=value]",
            "delete: accounts[id=value]",
            "update: accounts[id=#1] orders[id=value account_id=#0]",
        ]
    );
}

#[test]
fn positional_alias_facts_preserve_syntactic_pins() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/column-alias-list.sql"
    ));
    assert_eq!(
        shape(sql),
        [
            "select: accounts[id=value]",
            "select: accounts[id=value]",
            "select: accounts[id=#1] orders[account_id=#0 id=value]",
            "select: accounts[id=#1] orders[id=#0 id=value]",
            "select: accounts[id=(orders)]",
        ]
    );
}

#[test]
fn table_arms_use_the_recovered_source_spelling() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-quoting.sql"
    ));
    assert_eq!(
        shape(sql),
        [
            "select: () (\"Order Items\")",
            "select: () (public.\"Order Items\")",
            "select: () (())",
            "select: () (accounts)",
            "select: () (\"Accounts\")",
            "select: () (accounts)",
            "select: () (\"Accounts\")",
        ]
    );
}

#[test]
fn server_state_set_returning_functions_stay_opaque_in_from() {
    for call in [
        "pg_ls_dir('/tmp')",
        "pg_listening_channels()",
        "ts_stat($1)",
    ] {
        assert_eq!(shape(&format!("SELECT 1 FROM {call}")), ["select: opaque"]);
    }
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

#[test]
fn known_source_outputs_resolve_bare_columns_before_outer_pins() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projected-columns.sql"
    ));
    let shapes = shape(sql);
    assert!(shapes[..5].iter().all(|shape| shape == "delete: accounts"));
    assert!(shapes[5..].iter().all(|shape| shape.contains("[id=")));
}

#[test]
fn projected_source_aliases_set_arms_and_unknown_functions_preserve_ownership() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projected-column-variants.sql"));
    let shapes = shape(sql);
    assert_eq!(shapes.len(), 12);
    assert!(shapes.iter().all(|shape| shape.contains("[id=")));
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
fn unnest_without_a_column_alias_exposes_its_function_or_alias_name() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projected-unnest.sql"
    ));
    let shapes = shape(sql);
    assert_eq!(shapes.len(), 6);
    assert!(shapes[..4].iter().all(|shape| shape == "delete: accounts"));
    assert!(shapes[4..].iter().all(|shape| shape.contains("[id=")));
}

#[test]
fn qualified_scalar_functions_and_unknown_record_layouts_preserve_ownership() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projected-column-coverage.sql"));
    let shapes = shape(sql);
    assert_eq!(shapes.len(), 3);
    assert_eq!(shapes[0], "delete: accounts");
    assert!(shapes[1..].iter().all(|shape| shape.contains("[id=")));
}
