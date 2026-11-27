use super::{extract_sql_statement_facts, SqlCursorBound};

#[test]
fn cast_context_tracks_each_declared_bind_and_tuple_identity() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/cast-context-keysets/sql/pages.sql"));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.sweeps.len(), 21);
    for (index, sweep) in facts.sweeps.iter().enumerate() {
        let cursor = &sweep.conjuncts[0];
        let expected = if index < 8 {
            vec!["a".to_string(), "b".to_string()]
        } else {
            vec![]
        };
        assert_eq!(cursor.cursor_columns, expected, "statement {index}");
        assert_eq!(
            cursor.cursor_bound,
            (index < 8).then_some(SqlCursorBound::Lower),
            "statement {index}"
        );
    }
}

#[test]
fn recovered_cast_context_keeps_interpolation_provenance() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/cast-context-keysets/sql/recovered.sql"));
    let positions: Vec<_> = sql
        .match_indices("sql_placeholder_")
        .map(|(offset, _)| {
            let prefix = &sql[..offset];
            (
                prefix.bytes().filter(|byte| *byte == b'\n').count() as u32 + 1,
                prefix.rsplit('\n').next().unwrap().chars().count() as u32 + 1,
            )
        })
        .collect();
    let facts =
        super::extract_sql_statement_facts_with_recovered_placeholders(sql, true, &positions);
    assert!(!facts.parse_failed);
    assert_eq!(facts.sweeps.len(), 3);
    assert_eq!(
        facts
            .sweeps
            .iter()
            .map(|sweep| sweep.conjuncts[0].cursor_columns.clone())
            .collect::<Vec<_>>(),
        [
            vec!["a".to_string(), "b".to_string()],
            vec![],
            vec!["a".to_string(), "b".to_string()]
        ]
    );
    let standalone = extract_sql_statement_facts(sql);
    assert!(standalone
        .sweeps
        .iter()
        .all(|sweep| sweep.conjuncts[0].cursor_columns.is_empty()));
}
