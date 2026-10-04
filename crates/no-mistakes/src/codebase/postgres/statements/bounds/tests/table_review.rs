use super::shape;

#[test]
fn table_arms_preserve_both_possible_identifier_spellings() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-quoting.sql"
    ));
    assert_eq!(
        shape(sql),
        [
            "select: () (order items \"Order Items\")",
            "select: () (public.order items \"public\".\"Order Items\")",
            "select: () (order items ())",
            "select: () (accounts \"Accounts\")",
            "select: () (accounts \"Accounts\")",
            "select: () (accounts ())",
            "select: () (() \"Accounts\")",
        ]
    );
}
