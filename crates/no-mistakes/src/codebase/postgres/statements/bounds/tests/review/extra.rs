use super::super::shape;

#[test]
fn caller_sized_set_returning_functions_reject_server_derived_arguments() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/set-returning-arguments.sql"
    ));
    assert_eq!(
        shape(sql),
        [
            "select: opaque accounts[email=#0]",
            "select: opaque accounts[email=#0]",
            "select: opaque accounts[email=#0]",
            "select: opaque accounts[email=#0]",
            "select: other accounts[email=#0]",
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
fn catalog_set_returning_names_require_builtin_schema_identity() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/set-returning-qualified.sql"));
    assert_eq!(
        shape(sql),
        [
            "select: orders opaque",
            "select: orders opaque",
            "select: orders opaque"
        ]
    );
}

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
