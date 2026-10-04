use super::shape;

#[test]
fn standalone_table_queries_emit_bounded_facts() {
    let unaffected = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-nonquery-control.sql"
    ));
    assert_eq!(shape(unaffected), ["select: accounts"]);

    let bare = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-bare.sql"
    ));
    assert_eq!(shape(bare), ["select: accounts"]);

    let with_ddl = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-temporary.sql"
    ));
    assert_eq!(
        shape(with_ddl),
        [
            "select: opaque",
            "select: accounts",
            "select: public.\"Accounts\"",
            "select: public.accounts",
            "select: opaque",
            "select: opaque"
        ]
    );

    let recovered = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-lenient.sql"
    ));
    assert_eq!(shape(recovered), ["select: opaque", "select: accounts"]);
}
