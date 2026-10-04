use super::rebase_embedded_lines;
use crate::codebase::postgres::embedded::{
    EmbeddedSqlCall, EmbeddedSqlKind, EmbeddedSqlSourcePosition,
};
use crate::codebase::postgres::statements::extract_sql_statement_facts;

#[test]
fn embedded_line_shift_covers_returning_stars_and_triggers() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/line-rebase.sql");
    let sql = std::fs::read_to_string(path).unwrap();
    let mut facts = extract_sql_statement_facts(&sql);
    assert_eq!(facts.triggers.len(), 1, "{facts:?}");
    assert_eq!(facts.returning_stars.len(), 1, "{facts:?}");
    assert_eq!(
        facts
            .selects
            .iter()
            .flat_map(|s| &s.exists_set_operations)
            .count(),
        1
    );
    assert_eq!(facts.mutation_column_uses.len(), 1);
    rebase_embedded_lines(
        &mut facts,
        &EmbeddedSqlCall {
            line: 5,
            callee: "query".to_string(),
            sql_text: Some(sql),
            kind: EmbeddedSqlKind::Inline,
            declaration_line: None,
            sql_source_positions: Vec::new(),
            recovered_placeholder_positions: Vec::new(),
        },
    );
    assert_eq!(facts.origin_line, 5);
    assert!(facts.triggers[0].line > 1);
    assert!(facts.returning_stars[0].line > 1);
    assert_eq!(
        facts
            .selects
            .iter()
            .flat_map(|s| &s.exists_set_operations)
            .next()
            .unwrap()
            .line,
        7
    );
    assert_eq!(facts.mutation_column_uses[0].line, 11);
}

#[test]
fn bound_relations_rebase_by_their_sql_column() {
    // Separate source operands can share one SQL line: the column picks the owning operand.
    let sql = "SELECT * FROM accounts, orders".to_string();
    let mut facts = extract_sql_statement_facts(&sql);
    let position = |sql_column, source_line| EmbeddedSqlSourcePosition {
        sql_line: 1,
        sql_column,
        source_line,
    };
    let call = EmbeddedSqlCall {
        line: 10,
        callee: "query".to_string(),
        sql_text: Some(sql),
        kind: EmbeddedSqlKind::Inline,
        declaration_line: None,
        sql_source_positions: vec![position(1, 10), position(15, 20), position(25, 30)],
        recovered_placeholder_positions: Vec::new(),
    };
    rebase_embedded_lines(&mut facts, &call);
    let bound = &facts.bounds[0];
    assert_eq!(bound.line, 10);
    let lines: Vec<usize> = bound.query.items.iter().map(|item| item.line).collect();
    assert_eq!(lines, [20, 30]);
}

#[test]
fn conditional_lifecycle_bounds_follow_embedded_source_lines() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-shadow-drop.sql"
    ));
    let mut facts = extract_sql_statement_facts(sql);
    assert!(facts.lifecycle.is_some());
    rebase_embedded_lines(
        &mut facts,
        &EmbeddedSqlCall {
            line: 40,
            callee: "query".to_string(),
            sql_text: Some(sql.to_string()),
            kind: EmbeddedSqlKind::Inline,
            declaration_line: None,
            sql_source_positions: Vec::new(),
            recovered_placeholder_positions: Vec::new(),
        },
    );
    let lifecycle = facts.lifecycle.as_ref().unwrap();
    assert_eq!(lifecycle.raw_bounds.len(), facts.bounds.len());
    assert!(lifecycle.raw_bounds.iter().all(|bound| bound.line >= 40));
    assert!(lifecycle
        .raw_bounds
        .iter()
        .all(|bound| bound.statement_start.is_some_and(|(line, _)| line >= 40)));
    assert_eq!(
        lifecycle
            .raw_bounds
            .iter()
            .map(|bound| bound.statement_start)
            .collect::<Vec<_>>(),
        facts
            .bounds
            .iter()
            .map(|bound| bound.statement_start)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        lifecycle
            .raw_bounds
            .iter()
            .map(|bound| bound.line)
            .collect::<Vec<_>>(),
        facts
            .bounds
            .iter()
            .map(|bound| bound.line)
            .collect::<Vec<_>>()
    );
}
