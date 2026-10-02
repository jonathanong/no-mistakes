use super::rebase_embedded_lines;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlKind};
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
