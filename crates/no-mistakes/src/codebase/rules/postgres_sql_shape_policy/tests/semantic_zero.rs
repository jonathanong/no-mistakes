use super::bounded_iteration::{at, found};

#[test]
fn transparent_zero_counts_skip_keyset_sweeps_without_reclassifying_literals() {
    // This rule helper returns raw findings; request-level suppression is tested separately.
    assert_eq!(
        found(
            "semantic-zero",
            "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [literal-limit, keyset-only-sweep]\n",
            "sql/pages.sql"
        ),
        at(&[
            (6, "literal-limit"),
            (23, "keyset-only-sweep"),
            (24, "keyset-only-sweep"),
            (25, "keyset-only-sweep"),
            (26, "keyset-only-sweep"),
            (28, "keyset-only-sweep"),
            (29, "literal-limit"),
            (31, "keyset-only-sweep")
        ])
    );
}
