use super::bounded_iteration::{at, found};

#[test]
fn negated_comparisons_are_cursors_but_opposite_bounds_form_windows() {
    assert_eq!(
        found(
            "negated-keysets",
            "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [keyset-only-sweep]\n",
            "sql/pages.sql"
        ),
        at(&[
            (2, "keyset-only-sweep"),
            (3, "keyset-only-sweep"),
            (4, "keyset-only-sweep"),
            (5, "keyset-only-sweep"),
            (6, "keyset-only-sweep")
        ])
    );
}

#[test]
fn optional_cursors_survive_even_negation_parity() {
    assert_eq!(
        found(
            "negated-keysets",
            "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [keyset-only-sweep]\n",
            "sql/parity.sql"
        ),
        at(&[
            (2, "keyset-only-sweep"),
            (3, "keyset-only-sweep"),
            (5, "keyset-only-sweep"),
            (6, "keyset-only-sweep")
        ])
    );
}
