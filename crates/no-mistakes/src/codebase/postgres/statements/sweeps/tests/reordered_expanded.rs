use super::{extract_sql_statement_facts, SqlCursorBound};

#[test]
fn saved_expanded_cursors_ignore_or_arm_order_but_keep_the_full_chain() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/reordered-expanded-keysets/sql/pages.sql"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let cursors: Vec<_> = facts
        .sweeps
        .iter()
        .map(|sweep| {
            (
                sweep.line,
                sweep.conjuncts[0].cursor_columns.clone(),
                sweep.conjuncts[0].cursor_bound,
            )
        })
        .collect();
    assert_eq!(
        cursors,
        vec![
            (2, vec!["a".into(), "b".into()], Some(SqlCursorBound::Lower)),
            (3, vec!["a".into(), "b".into()], Some(SqlCursorBound::Lower)),
            (5, vec![], None),
            (6, vec![], None),
            (7, vec![], None),
        ]
    );
}
