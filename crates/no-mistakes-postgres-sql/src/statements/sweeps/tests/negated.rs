use super::{extract_sql_statement_facts, SqlCursorBound};

#[test]
fn saved_negated_keysets_reverse_cursor_bounds() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/negated-keysets/sql/pages.sql"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let bounds: Vec<_> = facts
        .sweeps
        .iter()
        .take(5)
        .map(|sweep| (sweep.line, sweep.conjuncts[0].cursor_bound))
        .collect();
    assert_eq!(
        bounds,
        vec![
            (2, Some(SqlCursorBound::Lower)),
            (3, Some(SqlCursorBound::Upper)),
            (4, Some(SqlCursorBound::Lower)),
            (5, Some(SqlCursorBound::Lower)),
            (6, Some(SqlCursorBound::Lower))
        ]
    );
    assert_eq!(
        facts.sweeps[4].conjuncts[0].cursor_columns,
        vec!["id", "created_at"]
    );
}

#[test]
fn saved_even_negations_preserve_optional_cursors() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/negated-keysets/sql/parity.sql"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(
        facts
            .sweeps
            .iter()
            .map(|s| (s.conjuncts[0].cursor_bound, s.conjuncts[0].cursor_optional))
            .collect::<Vec<_>>(),
        vec![
            (Some(SqlCursorBound::Lower), true),
            (Some(SqlCursorBound::Lower), true),
            (None, false),
            (Some(SqlCursorBound::Lower), false),
            (Some(SqlCursorBound::Upper), false),
            (None, false),
        ]
    );
}
