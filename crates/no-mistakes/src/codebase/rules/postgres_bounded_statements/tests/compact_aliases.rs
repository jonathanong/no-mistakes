use super::unbounded;

#[test]
fn oversized_aliased_ctes_keep_the_nonunique_read_unbounded() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/oversized-cte-aliased-pins.sql"));
    assert_eq!(
        unbounded(sql),
        [
            ("orders".into(), 2),
            ("orders".into(), 38),
            ("orders".into(), 56),
            ("orders".into(), 74)
        ]
    );
}
