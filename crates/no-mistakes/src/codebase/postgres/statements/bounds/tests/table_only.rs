use super::shape;

#[test]
fn table_only_arms_preserve_quoted_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-only.sql"
    ));
    assert_eq!(
        shape(sql),
        [
            "select: () (opaque)",
            "select: () (accounts)",
            "select: () (public.\"Accounts\")"
        ]
    );
}
