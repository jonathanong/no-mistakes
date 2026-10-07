use super::super::PostgresSqlStatementKind;
use super::{facts, fixture};

#[test]
fn escape_continuations_inherit_quote_state_and_keep_original_spans() {
    let sql = fixture("escaped-continuations.sql");
    let result = facts("escaped-continuations.sql");
    assert_eq!(result.diagnostics.len(), 5, "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 6);
    for (statement, expected) in result.statements.iter().zip([
        "foobar",
        "first\nsecond\tend\r\u{0008}\u{000c}\\z",
        "ABCDxZ",
        "é雪",
    ]) {
        let PostgresSqlStatementKind::Comment { comment } = &statement.facts else {
            panic!()
        };
        assert_eq!(comment.comment.as_deref(), Some(expected));
        assert_eq!(
            statement.sql,
            sql[statement.span.start.offset..statement.span.end.offset]
        );
    }
    assert!(matches!(
        result.statements[4].facts,
        PostgresSqlStatementKind::CreateTable { .. }
    ));
    assert!(matches!(
        result.statements[5].facts,
        PostgresSqlStatementKind::Insert { .. }
    ));
}
