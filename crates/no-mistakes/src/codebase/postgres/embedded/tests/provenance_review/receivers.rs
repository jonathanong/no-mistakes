use super::*;

#[test]
fn unescaped_typed_and_factory_receivers_are_positive_controls() {
    assert_sites(
        "review-positive-controls.ts",
        &scoped_options(),
        &[
            Site {
                line: 5,
                expression: "tx.query('SELECT id FROM control_typed')",
                sql: Some("SELECT id FROM control_typed"),
                kind: EmbeddedSqlKind::Inline,
            },
            Site {
                line: 10,
                expression: "tx.query('SELECT id FROM control_factory')",
                sql: Some("SELECT id FROM control_factory"),
                kind: EmbeddedSqlKind::Inline,
            },
        ],
    );
}

#[test]
fn typed_and_factory_escapes_preserve_independent_and_prior_calls() {
    use EmbeddedSqlKind::{Dynamic, Inline};
    assert_sites(
        "review-scoped-receivers.ts",
        &scoped_options(),
        &[
            Site {
                line: 6,
                expression: "first.query('SELECT id FROM typed_before')",
                sql: Some("SELECT id FROM typed_before"),
                kind: Inline,
            },
            Site {
                line: 9,
                expression: "first.query('SELECT id FROM typed_after')",
                sql: None,
                kind: Dynamic,
            },
            Site {
                line: 10,
                expression: "alias.query('SELECT id FROM typed_alias_after')",
                sql: None,
                kind: Dynamic,
            },
            Site {
                line: 11,
                expression: "other.query('SELECT id FROM typed_other')",
                sql: Some("SELECT id FROM typed_other"),
                kind: Inline,
            },
            Site {
                line: 18,
                expression: "first.query('SELECT id FROM factory_before')",
                sql: Some("SELECT id FROM factory_before"),
                kind: Inline,
            },
            Site {
                line: 20,
                expression: "first.query('SELECT id FROM factory_after')",
                sql: None,
                kind: Dynamic,
            },
            Site {
                line: 21,
                expression: "other.query('SELECT id FROM factory_other')",
                sql: Some("SELECT id FROM factory_other"),
                kind: Inline,
            },
        ],
    );
}

#[test]
fn same_spelling_in_sibling_functions_has_independent_proof() {
    use EmbeddedSqlKind::{Dynamic, Inline};
    assert_sites(
        "review-sibling-scopes.ts",
        &scoped_options(),
        &[
            Site {
                line: 6,
                expression: "tx.query('SELECT id FROM sibling_escaped')",
                sql: None,
                kind: Dynamic,
            },
            Site {
                line: 10,
                expression: "tx.query('SELECT id FROM sibling_untouched')",
                sql: Some("SELECT id FROM sibling_untouched"),
                kind: Inline,
            },
        ],
    );
}
