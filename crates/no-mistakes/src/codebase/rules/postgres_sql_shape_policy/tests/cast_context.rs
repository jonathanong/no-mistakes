use super::bounded_iteration::{at, found};

#[test]
fn cast_context_preserves_public_keyset_findings() {
    let config = "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [keyset-only-sweep]\n";
    assert_eq!(
        found("cast-context-keysets", config, "sql/pages.sql"),
        at(&[
            (2, "keyset-only-sweep"),
            (3, "keyset-only-sweep"),
            (4, "keyset-only-sweep"),
            (5, "keyset-only-sweep"),
            (6, "keyset-only-sweep"),
            (7, "keyset-only-sweep"),
            (8, "keyset-only-sweep"),
            (9, "keyset-only-sweep")
        ])
    );
}
