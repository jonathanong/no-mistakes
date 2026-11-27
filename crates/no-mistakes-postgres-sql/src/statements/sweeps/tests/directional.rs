use super::{extract_sql_statement_facts, SqlCursorBound};

#[test]
fn saved_expanded_cursor_directions_and_final_inclusivity() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/directional-keysets/sql/pages.sql"));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.sweeps.len(), 20);
    for (index, sweep) in facts.sweeps.iter().enumerate() {
        let cursor = &sweep.conjuncts[0];
        let count = if index == 6 {
            3
        } else if index == 9 {
            1
        } else {
            2
        };
        let expected: Vec<_> = if index < 10 {
            ["a", "b", "c"][..count]
                .iter()
                .map(|name| name.to_string())
                .collect()
        } else {
            vec![]
        };
        assert_eq!(cursor.cursor_columns, expected, "statement {index}");
        let bound = match index {
            1 | 3 | 4 => Some(SqlCursorBound::Upper),
            0..=9 => Some(SqlCursorBound::Lower),
            _ => None,
        };
        assert_eq!(cursor.cursor_bound, bound, "statement {index}");
    }
}
