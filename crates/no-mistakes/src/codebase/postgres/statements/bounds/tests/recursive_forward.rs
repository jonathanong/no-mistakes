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
