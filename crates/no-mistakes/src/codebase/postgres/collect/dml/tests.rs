use super::rebase_embedded_lines;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlKind};
use crate::codebase::postgres::statements::extract_sql_statement_facts;

#[test]
fn embedded_line_shift_covers_returning_stars_and_triggers() {
    let sql = "CREATE TRIGGER touch AFTER INSERT ON orders FOR EACH ROW EXECUTE FUNCTION touch();\nINSERT INTO orders (id) VALUES ($1) RETURNING *;";
    let mut facts = extract_sql_statement_facts(sql);
    assert_eq!(facts.triggers.len(), 1, "{facts:?}");
    assert_eq!(facts.returning_stars.len(), 1, "{facts:?}");
    rebase_embedded_lines(
        &mut facts,
        &EmbeddedSqlCall {
            line: 5,
            callee: "query".to_string(),
            sql_text: Some(sql.to_string()),
            kind: EmbeddedSqlKind::Inline,
            declaration_line: None,
        },
    );
    assert_eq!(facts.origin_line, 5);
    assert!(facts.triggers[0].line > 1);
    assert!(facts.returning_stars[0].line > 1);
}
