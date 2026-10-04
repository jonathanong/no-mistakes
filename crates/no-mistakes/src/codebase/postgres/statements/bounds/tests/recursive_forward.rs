use super::shape;

#[test]
fn recursive_forward_ctes_preserve_the_catalog_leaf() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/recursive-forward-bounds.sql"
    ));
    assert_eq!(
        shape(sql),
        [
            "select: ((public.accounts))",
            "select: (((public.accounts)))"
        ]
    );
}

#[test]
fn recursive_table_arms_parse_and_preserve_source_identity() {
    for (fixture, expected) in [
        ("recursive-table-forward.sql", "select: ((public.accounts))"),
        (
            "recursive-table-quoted-union.sql",
            "select: ((\"billing.accounts\") ((public.accounts)))",
        ),
        (
            "recursive-table-qualified-union.sql",
            "select: ((billing.accounts) ((public.accounts)))",
        ),
    ] {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/fixture/sql");
        let sql = std::fs::read_to_string(root.join(fixture)).unwrap();
        let parsed = crate::codebase::postgres::parse_postgres_sql(&sql)
            .unwrap_or_else(|error| panic!("{fixture}: {error}"));
        assert_eq!(parsed.len(), 1, "{fixture}");
        assert_eq!(shape(&sql), [expected], "{fixture}");
    }
}
