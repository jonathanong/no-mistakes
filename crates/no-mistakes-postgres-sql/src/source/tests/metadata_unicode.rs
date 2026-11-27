use super::super::PostgresSqlStatementKind;
use super::{facts, fixture};

#[test]
fn unicode_comments_decode_real_escape_spelling_and_recover_invalid_controls() {
    let sql = fixture("unicode-comment-escapes.sql");
    let result = facts("unicode-comment-escapes.sql");
    assert_eq!(result.diagnostics.len(), 13, "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 21, "{:?}", result.statements);
    for (statement, expected) in result.statements.iter().take(7).zip([
        Some("data! it''s \\path"),
        Some("snow 雪"),
        Some("data\\"),
        Some("😀 😀 😀 😀"),
        Some("a§"),
        None,
        Some("!0041\\0041"),
    ]) {
        let PostgresSqlStatementKind::Comment { comment } = &statement.facts else {
            panic!()
        };
        assert_eq!(comment.comment.as_deref(), expected);
        assert_eq!(
            statement.sql,
            sql[statement.span.start.offset..statement.span.end.offset]
        );
    }
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[7].facts else {
        panic!()
    };
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!()
    };
    assert_eq!(branches[0].statements.len(), 2);
    for (statement, expected) in branches[0].statements.iter().zip(["A", "B"]) {
        let PostgresSqlStatementKind::Comment { comment } = &statement.facts else {
            panic!()
        };
        assert_eq!(comment.comment.as_deref(), Some(expected));
    }
    assert!(result.statements[8..]
        .iter()
        .all(|s| matches!(s.facts, PostgresSqlStatementKind::Select { .. })));
}
